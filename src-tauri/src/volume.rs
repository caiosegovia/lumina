use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeIdentity {
    pub id: String,
    pub label: String,
}

#[cfg(windows)]
pub fn identify(path: &Path) -> Result<VolumeIdentity, String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        GetVolumeInformationW, GetVolumeNameForVolumeMountPointW, GetVolumePathNameW,
    };
    let input = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut root = vec![0u16; 261];
    if unsafe { GetVolumePathNameW(input.as_ptr(), root.as_mut_ptr(), root.len() as u32) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let mut guid = vec![0u16; 261];
    if unsafe {
        GetVolumeNameForVolumeMountPointW(root.as_ptr(), guid.as_mut_ptr(), guid.len() as u32)
    } == 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let mut serial = 0u32;
    let mut label = vec![0u16; 261];
    if unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            label.as_mut_ptr(),
            label.len() as u32,
            &mut serial,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let decode = |buffer: &[u16]| {
        String::from_utf16_lossy(
            &buffer[..buffer
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(buffer.len())],
        )
    };
    let guid = decode(&guid);
    let label = decode(&label);
    Ok(VolumeIdentity {
        id: format!("{guid}:{serial:08X}"),
        label: if label.is_empty() {
            decode(&root)
        } else {
            label
        },
    })
}

#[cfg(not(windows))]
pub fn identify(path: &Path) -> Result<VolumeIdentity, String> {
    let value = path
        .components()
        .next()
        .map(|x| x.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "/".into());
    Ok(VolumeIdentity {
        id: value.clone(),
        label: value,
    })
}

pub fn source_key(identity: &VolumeIdentity, path: &Path) -> String {
    format!("{}::{}", identity.id, path.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_mount_paths_on_distinct_volumes_have_distinct_keys() {
        let path = Path::new("F:/DCIM");
        let a = VolumeIdentity {
            id: "a".into(),
            label: "A".into(),
        };
        let b = VolumeIdentity {
            id: "b".into(),
            label: "B".into(),
        };
        assert_ne!(source_key(&a, path), source_key(&b, path));
    }
    #[test]
    #[cfg(windows)]
    fn windows_volume_identity_is_stable() {
        let first = identify(&std::env::temp_dir()).unwrap();
        let second = identify(&std::env::temp_dir()).unwrap();
        assert!(!first.id.is_empty());
        assert_eq!(first, second);
    }
}
