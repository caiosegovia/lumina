use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};
pub fn sha256(path: &Path) -> Result<String, String> {
    sha256_cancel(path, None)
}
pub fn sha256_cancel(
    path: &Path,
    cancel: Option<&crate::process::CancellationToken>,
) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; 8 * 1024 * 1024];
    loop {
        if cancel.is_some_and(|token| token.is_cancelled()) {
            return Err("JOB_CANCELED".into());
        }
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read])
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn copy_verified(source: &Path, destination: &Path, expected: &str) -> Result<(), String> {
    let temp = destination.with_extension(format!(
        "{}.lumina-part",
        destination
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("tmp")
    ));
    copy_verified_via(source, destination, &temp, expected)
}
pub fn copy_verified_via(
    source: &Path,
    destination: &Path,
    temp: &Path,
    expected: &str,
) -> Result<(), String> {
    copy_verified_via_staged(source, destination, temp, expected, |_| {})
}
pub fn copy_verified_via_staged<F: FnMut(&str)>(
    source: &Path,
    destination: &Path,
    temp: &Path,
    expected: &str,
    mut on_stage: F,
) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    if destination.exists() && sha256(destination)? == expected {
        let _ = fs::remove_file(temp);
        return Ok(());
    }
    let mut input = File::open(source).map_err(|e| e.to_string())?;
    let mut output = File::create(temp).map_err(|e| e.to_string())?;
    std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
    output.flush().map_err(|e| e.to_string())?;
    output.sync_all().map_err(|e| e.to_string())?;
    on_stage("verification");
    if sha256(temp)? != expected {
        let _ = fs::remove_file(temp);
        return Err("A verificação SHA-256 da cópia falhou".into());
    }
    if destination.exists() {
        if sha256(destination)? == expected {
            let _ = fs::remove_file(temp);
            return Ok(());
        }
        return Err("O destino já contém outro arquivo".into());
    }
    on_stage("promotion");
    fs::rename(temp, destination).map_err(|e| e.to_string())
}
pub fn copy_hash_to_temp_verified(
    source: &Path,
    temp: &Path,
    cancel: Option<&crate::process::CancellationToken>,
) -> Result<String, String> {
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    let mut input = File::open(source).map_err(|e| e.to_string())?;
    let mut output = File::create(temp).map_err(|e| e.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; 8 * 1024 * 1024];
    loop {
        if cancel.is_some_and(|x| x.is_cancelled()) {
            let _ = fs::remove_file(temp);
            return Err("JOB_CANCELED".into());
        }
        let read = input.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        output
            .write_all(&buffer[..read])
            .map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())?;
    output.sync_all().map_err(|e| e.to_string())?;
    let source_hash = format!("{:x}", digest.finalize());
    if sha256_cancel(temp, cancel)? != source_hash {
        let _ = fs::remove_file(temp);
        return Err("A verificação SHA-256 da cópia falhou".into());
    }
    Ok(source_hash)
}
pub fn promote_verified_temp(
    temp: &Path,
    destination: &Path,
    expected: &str,
) -> Result<(), String> {
    if sha256(temp)? != expected {
        return Err("O temporário verificado foi alterado".into());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    if destination.exists() {
        if sha256(destination)? == expected {
            let _ = fs::remove_file(temp);
            return Ok(());
        }
        return Err("O destino já contém outro arquivo".into());
    }
    fs::rename(temp, destination).map_err(|e| e.to_string())
}
pub fn safe_destination(
    dir: &Path,
    filename: &str,
    source: &Path,
    hash: &str,
) -> Result<std::path::PathBuf, String> {
    let mut destination = dir.join(filename);
    if destination.exists() && sha256(&destination).ok().as_deref() != Some(hash) {
        let stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("media");
        let extension = source.extension().and_then(|s| s.to_str()).unwrap_or("bin");
        destination = dir.join(format!("{}-{}.{}", stem, &hash[..8], extension))
    }
    Ok(destination)
}
#[cfg(test)]
pub fn ensure_space(required: u64, available: u64) -> Result<(), String> {
    if required > available {
        Err(format!("Espaço insuficiente: são necessários {required} bytes e existem {available} bytes livres"))
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn destination_collision_is_stable() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("same.jpg");
        fs::write(root.join("x.jpg"), b"a").unwrap();
        fs::write(&source, b"b").unwrap();
        let result = safe_destination(&root, "x.jpg", &source, &sha256(&source).unwrap()).unwrap();
        let expected = format!("same-{}.jpg", &sha256(&source).unwrap()[..8]);
        assert_eq!(
            result.file_name().and_then(|value| value.to_str()),
            Some(expected.as_str())
        );
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn rejects_insufficient_space_before_copy() {
        let error = ensure_space(101, 100).unwrap_err();
        assert!(error.contains("101"));
        assert!(ensure_space(100, 100).is_ok());
    }
    #[test]
    fn hashes_while_copying_and_promotes_only_verified_bytes() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("source.bin");
        let temp = root.join("temp.part");
        let destination = root.join("final.bin");
        fs::write(&source, vec![7u8; 1024 * 1024]).unwrap();
        let hash = copy_hash_to_temp_verified(&source, &temp, None).unwrap();
        assert_eq!(hash, sha256(&source).unwrap());
        promote_verified_temp(&temp, &destination, &hash).unwrap();
        assert_eq!(fs::read(destination).unwrap(), fs::read(source).unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
