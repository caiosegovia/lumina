use chrono::Utc;
use fs2::FileExt;
use rusqlite::params;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};
use uuid::Uuid;
pub struct LibraryLock {
    file: File,
}
impl LibraryLock {
    pub fn acquire(master: &Path, instance: &str) -> Result<Self, String> {
        let dir = master.join(".lumina");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("library.lock");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.try_lock_exclusive().map_err(|_|"Esta biblioteca já está aberta por outra instância do Lumina. Feche a outra instância antes de continuar.".to_string())?;
        file.set_len(0).map_err(|e| e.to_string())?;
        write!(file, "instance={instance}\npid={}\n", std::process::id())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        Ok(Self { file })
    }
}
impl Drop for LibraryLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
pub fn migrate_master(
    cfg: &crate::models::LibraryConfig,
    new_master: &Path,
) -> Result<crate::models::MigrationProgress, String> {
    fs::create_dir_all(new_master)
        .map_err(|e| format!("Não foi possível preparar o novo acervo: {e}"))?;
    let old = fs::canonicalize(&cfg.master_path).map_err(|e| e.to_string())?;
    let new_canonical = fs::canonicalize(new_master).map_err(|e| e.to_string())?;
    if old == new_canonical || old.starts_with(&new_canonical) || new_canonical.starts_with(&old) {
        return Err("O novo acervo precisa estar fora do acervo atual".into());
    }
    let new = new_master.to_path_buf();
    let db_path = old.join(".lumina/catalog.sqlite");
    let conn = crate::catalog::open(&db_path).map_err(|e| e.to_string())?;
    let (total_items, total_bytes): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*),COALESCE(SUM(bytes),0)FROM assets",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let available = fs2::available_space(&new).map_err(|e| e.to_string())?;
    let reserve = 1024u64 * 1024 * 1024;
    if total_bytes.max(0) as u64 + reserve > available {
        return Err(format!(
            "O novo acervo precisa de {} bytes e possui {} bytes livres",
            total_bytes, available
        ));
    }
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT INTO library_migrations(id,old_master,new_master,state,total_items,total_bytes,created_at,updated_at)VALUES(?1,?2,?3,'copying',?4,?5,?6,?6)",params![id,old.to_string_lossy(),new.to_string_lossy(),total_items,total_bytes,now]).map_err(|e|e.to_string())?;
    let rows = {
        let mut s = conn
            .prepare("SELECT id,master_path,hash,bytes FROM assets ORDER BY id")
            .map_err(|e| e.to_string())?;
        let values = s
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let mut processed = 0i64;
    let mut processed_bytes = 0i64;
    for (_, path, hash, bytes) in &rows {
        let source =
            fs::canonicalize(path).map_err(|e| format!("Não foi possível acessar {path}: {e}"))?;
        let rel = source
            .strip_prefix(&old)
            .map_err(|_| format!("Arquivo fora do acervo atual: {path}"))?;
        let destination = new.join(rel);
        if let Err(error) = crate::storage::copy_verified(&source, &destination, hash) {
            conn.execute("UPDATE library_migrations SET state='failed',last_error=?2,updated_at=?3 WHERE id=?1",params![id,error,Utc::now().to_rfc3339()]).ok();
            return Err(error);
        }
        processed += 1;
        processed_bytes += *bytes;
        conn.execute("UPDATE library_migrations SET processed_items=?2,processed_bytes=?3,updated_at=?4 WHERE id=?1",params![id,processed,processed_bytes,Utc::now().to_rfc3339()]).ok();
    }
    conn.execute(
        "UPDATE library_migrations SET state='verified',updated_at=?2 WHERE id=?1",
        params![id, Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA wal_checkpoint(FULL)").ok();
    fs::create_dir_all(new.join(".lumina")).map_err(|e| e.to_string())?;
    fs::copy(&db_path, new.join(".lumina/catalog.sqlite")).map_err(|e| e.to_string())?;
    let target =
        crate::catalog::open(&new.join(".lumina/catalog.sqlite")).map_err(|e| e.to_string())?;
    for (asset, path, _, _) in rows {
        let source = fs::canonicalize(&path).map_err(|e| e.to_string())?;
        let rel = source.strip_prefix(&old).map_err(|e| e.to_string())?;
        target
            .execute(
                "UPDATE assets SET master_path=?2 WHERE id=?1",
                params![asset, new.join(rel).to_string_lossy()],
            )
            .map_err(|e| e.to_string())?;
    }
    target.execute("UPDATE thumbnails SET state='stale',path='',last_error='Acervo migrado; miniatura será reconstruída',updated_at=?1",[Utc::now().to_rfc3339()]).ok();
    target
        .execute(
            "UPDATE library_migrations SET state='completed',updated_at=?2 WHERE id=?1",
            params![id, Utc::now().to_rfc3339()],
        )
        .ok();
    target.execute_batch("PRAGMA wal_checkpoint(FULL)").ok();
    Ok(crate::models::MigrationProgress {
        id,
        old_master: old.to_string_lossy().into(),
        new_master: new.to_string_lossy().into(),
        state: "completed".into(),
        processed_items: processed,
        total_items,
        processed_bytes,
        total_bytes,
        last_error: None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    #[test]
    fn prevents_two_writers() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let first = LibraryLock::acquire(&root, "one").unwrap();
        assert!(LibraryLock::acquire(&root, "two").is_err());
        drop(first);
        assert!(LibraryLock::acquire(&root, "three").is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn migration_copies_and_verifies_before_switching_catalog_paths() {
        // The catalog owns its metadata directory; production creates it during onboarding.
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let old = root.join("old");
        let new = root.join("new");
        let backup = root.join("backup");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let media = old.join("2026/01/photo.jpg");
        fs::create_dir_all(media.parent().unwrap()).unwrap();
        fs::write(&media, b"original bytes").unwrap();
        let hash = crate::storage::sha256(&media).unwrap();
        let cfg = crate::models::LibraryConfig {
            id: "l".into(),
            name: "test".into(),
            master_path: old.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = crate::catalog::open(&old.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'photo.jpg','photo','jpg',?2,'file',14,?3,?2)",params![hash,Utc::now().to_rfc3339(),media.to_string_lossy()]).unwrap();
        drop(conn);
        let result = migrate_master(&cfg, &new).unwrap();
        assert_eq!(result.state, "completed");
        assert_eq!(
            fs::read(new.join("2026/01/photo.jpg")).unwrap(),
            b"original bytes"
        );
        assert!(media.exists());
        let migrated = crate::catalog::open(&new.join(".lumina/catalog.sqlite")).unwrap();
        let path: String = migrated
            .query_row("SELECT master_path FROM assets WHERE id='a'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(Path::new(&path).starts_with(&new));
        drop(migrated);
        fs::remove_dir_all(root).unwrap();
    }
}
