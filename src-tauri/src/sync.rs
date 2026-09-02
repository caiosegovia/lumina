use crate::{catalog, models::*, process::CancellationToken, storage};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use std::{collections::HashSet, fs, path::Path};
use uuid::Uuid;
use walkdir::WalkDir;

fn modified(metadata: &fs::Metadata) -> String {
    metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(Utc::now)
        .to_rfc3339()
}

fn catalog_path(cfg: &LibraryConfig) -> std::path::PathBuf {
    Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")
}

pub fn queue(cfg: &LibraryConfig, source_id: &str) -> Result<String, String> {
    let conn = catalog::open(&catalog_path(cfg)).map_err(|error| error.to_string())?;
    let (path, available): (String, bool) = conn
        .query_row(
            "SELECT COALESCE(mount_path,path),available FROM sources WHERE id=?1 AND path NOT LIKE 'lumina://%'",
            [source_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Fonte não encontrada".to_string())?;
    if !available || !Path::new(&path).is_dir() {
        return Err("A fonte está offline ou inacessível".into());
    }
    let job = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO jobs(id,source_id,source_path,state,stage,job_kind,created_at,updated_at)VALUES(?1,?2,?3,'queued','sync_inventory','source_sync',?4,?4)",
        params![job, source_id, path, now],
    )
    .map_err(|error| error.to_string())?;
    conn.execute("INSERT INTO job_counters(job_id)VALUES(?1)", [&job])
        .map_err(|error| error.to_string())?;
    conn.execute("INSERT INTO source_sync_settings(source_id,last_state)VALUES(?1,'queued')ON CONFLICT(source_id)DO UPDATE SET last_state='queued',last_error=NULL",[source_id]).map_err(|error|error.to_string())?;
    Ok(job)
}

pub fn run(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &CancellationToken,
) -> Result<SourceSyncSummary, String> {
    let db_path = catalog_path(cfg);
    let mut conn = catalog::open(&db_path).map_err(|error| error.to_string())?;
    let (source_id, source_path): (String, String) = conn
        .query_row(
            "SELECT source_id,source_path FROM jobs WHERE id=?1 AND job_kind='source_sync'",
            [job],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Sincronização não encontrada".to_string())?;
    let root = Path::new(&source_path);
    if !root.is_dir() {
        conn.execute("UPDATE sources SET available=0 WHERE id=?1", [&source_id])
            .ok();
        return Err("A fonte ficou offline antes da sincronização".into());
    }
    let source_abs = fs::canonicalize(root).map_err(|error| error.to_string())?;
    for protected in [&cfg.master_path, &cfg.backup_path] {
        let protected = fs::canonicalize(protected).map_err(|error| error.to_string())?;
        if source_abs.starts_with(&protected) || protected.starts_with(&source_abs) {
            return Err("A sincronização não pode usar o acervo ou a réplica como fonte".into());
        }
    }

    let started_at = Utc::now().to_rfc3339();
    let mut files = Vec::new();
    let mut total_bytes = 0i64;
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let extension = entry
            .path()
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if crate::pipeline::MEDIA.contains(&extension.as_str()) {
            let metadata = entry.metadata().map_err(|error| error.to_string())?;
            total_bytes = total_bytes.saturating_add(metadata.len().min(i64::MAX as u64) as i64);
            files.push((entry.path().to_path_buf(), extension, metadata));
        }
    }
    conn.execute("UPDATE jobs SET state='analyzing',stage='sync_reconcile',started_at=?2,total_items=?3,total_bytes=?4,processed_items=0,processed_bytes=0,updated_at=?2 WHERE id=?1",params![job,started_at,files.len() as i64,total_bytes]).map_err(|error|error.to_string())?;
    conn.execute("INSERT INTO source_sync_settings(source_id,last_started_at,last_state,last_error)VALUES(?1,?2,'running',NULL)ON CONFLICT(source_id)DO UPDATE SET last_started_at=excluded.last_started_at,last_state='running',last_error=NULL",params![source_id,started_at]).map_err(|error|error.to_string())?;

    let mut summary = SourceSyncSummary {
        job_id: job.into(),
        source_id: source_id.clone(),
        discovered: files.len() as i64,
        present: 0,
        new_files: 0,
        duplicates: 0,
        changed: 0,
        missing: 0,
        failed: 0,
        processed_bytes: 0,
    };
    let mut seen = HashSet::new();
    for (path, extension, metadata) in files {
        if cancel.is_cancelled() {
            return Err("JOB_CANCELED".into());
        }
        let path_text = path.to_string_lossy().to_string();
        seen.insert(path_text.clone());
        let bytes = metadata.len().min(i64::MAX as u64) as i64;
        let modified_at = modified(&metadata);
        let prior: Option<(i64, String, Option<String>, Option<String>)> = conn
            .query_row(
                "SELECT bytes,modified_at,hash,asset_id FROM source_inventory WHERE source_id=?1 AND path=?2",
                params![source_id, path_text],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let unchanged = prior
            .as_ref()
            .is_some_and(|value| value.0 == bytes && value.1 == modified_at && value.2.is_some());
        let hash = if unchanged {
            prior.as_ref().and_then(|value| value.2.clone()).unwrap()
        } else {
            storage::sha256_cancel(&path, Some(cancel))?
        };
        let asset_id: Option<String> = conn
            .query_row("SELECT id FROM assets WHERE hash=?1", [&hash], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|error| error.to_string())?;
        let state = if let Some(asset) = asset_id.as_ref() {
            let occurrence = conn
                .query_row(
                    "SELECT id FROM occurrences WHERE source_id=?1 AND path=?2",
                    params![source_id, path_text],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .map_err(|error| error.to_string())?
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            conn.execute("INSERT INTO occurrences(id,asset_id,source_id,path,seen_at)VALUES(?1,?2,?3,?4,?5)ON CONFLICT(source_id,path)DO UPDATE SET asset_id=excluded.asset_id,seen_at=excluded.seen_at",params![occurrence,asset,source_id,path_text,started_at]).map_err(|error|error.to_string())?;
            conn.execute("INSERT INTO occurrence_presence(occurrence_id,state,last_seen_at,missing_since)VALUES(?1,'present',?2,NULL)ON CONFLICT(occurrence_id)DO UPDATE SET state='present',last_seen_at=excluded.last_seen_at,missing_since=NULL",params![occurrence,started_at]).map_err(|error|error.to_string())?;
            if unchanged {
                summary.present += 1;
                "present"
            } else {
                summary.duplicates += 1;
                "duplicate"
            }
        } else if prior
            .as_ref()
            .is_some_and(|value| value.2.as_deref() != Some(&hash))
        {
            summary.changed += 1;
            "changed"
        } else {
            summary.new_files += 1;
            "new"
        };
        conn.execute("INSERT INTO source_inventory(source_id,path,filename,extension,bytes,modified_at,hash,asset_id,state,last_seen_at,missing_since,last_error)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,NULL,NULL)ON CONFLICT(source_id,path)DO UPDATE SET filename=excluded.filename,extension=excluded.extension,bytes=excluded.bytes,modified_at=excluded.modified_at,hash=excluded.hash,asset_id=excluded.asset_id,state=excluded.state,last_seen_at=excluded.last_seen_at,missing_since=NULL,last_error=NULL",params![source_id,path_text,path.file_name().unwrap_or_default().to_string_lossy(),extension,bytes,modified_at,hash,asset_id,state,started_at]).map_err(|error|error.to_string())?;
        summary.processed_bytes = summary.processed_bytes.saturating_add(bytes);
        let processed = summary.present + summary.new_files + summary.duplicates + summary.changed;
        conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,current_file=?4,imported_count=?5,duplicate_count=?6,updated_at=?7 WHERE id=?1",params![job,processed,summary.processed_bytes,path_text,summary.new_files,summary.duplicates,Utc::now().to_rfc3339()]).map_err(|error|error.to_string())?;
    }

    let mut missing_statement = conn.prepare("SELECT path FROM source_inventory WHERE source_id=?1 AND last_seen_at<>?2 AND state<>'missing'").map_err(|error|error.to_string())?;
    let missing_paths = missing_statement
        .query_map(params![source_id, started_at], |row| {
            row.get::<_, String>(0)
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(missing_statement);
    summary.missing = missing_paths.len() as i64;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    for missing in missing_paths {
        transaction.execute("UPDATE source_inventory SET state='missing',missing_since=COALESCE(missing_since,?3) WHERE source_id=?1 AND path=?2",params![source_id,missing,started_at]).map_err(|error|error.to_string())?;
        transaction.execute("INSERT INTO occurrence_presence(occurrence_id,state,last_seen_at,missing_since)SELECT id,'missing',seen_at,?3 FROM occurrences WHERE source_id=?1 AND path=?2 ON CONFLICT(occurrence_id)DO UPDATE SET state='missing',missing_since=COALESCE(occurrence_presence.missing_since,excluded.missing_since)",params![source_id,missing,started_at]).map_err(|error|error.to_string())?;
    }
    let finished = Utc::now().to_rfc3339();
    transaction.execute("UPDATE sources SET available=1,last_scan=?2,asset_count=(SELECT COUNT(*) FROM source_inventory WHERE source_id=?1 AND state<>'missing') WHERE id=?1",params![source_id,finished]).map_err(|error|error.to_string())?;
    transaction.execute("UPDATE source_sync_settings SET last_completed_at=?2,last_state='completed',last_error=NULL WHERE source_id=?1",params![source_id,finished]).map_err(|error|error.to_string())?;
    transaction.execute("UPDATE jobs SET state='completed',stage='sync_completed',processed_items=total_items,processed_bytes=total_bytes,excluded_count=?2,finished_at=?3,updated_at=?3,current_file=NULL WHERE id=?1",params![job,summary.missing,finished]).map_err(|error|error.to_string())?;
    transaction
        .execute(
            "INSERT INTO events(job_id,at,path,state,details)VALUES(?1,?2,'','completed',?3)",
            params![
                job,
                finished,
                format!(
                    "Sincronização concluída: {} presentes, {} novas, {} duplicatas, {} ausentes",
                    summary.present, summary.new_files, summary.duplicates, summary.missing
                )
            ],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture() -> (std::path::PathBuf, LibraryConfig, String) {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        let source = root.join("source");
        fs::create_dir_all(master.join(".lumina")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        fs::create_dir_all(&source).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&catalog_path(&cfg)).unwrap();
        conn.execute("INSERT INTO sources(id,name,path,volume_label,mount_path)VALUES('s','Fonte','key','v',?1)",[source.to_string_lossy().as_ref()]).unwrap();
        (root, cfg, "s".into())
    }

    #[test]
    fn synchronization_discovers_duplicates_and_marks_missing_without_touching_source() {
        let (root, cfg, source_id) = fixture();
        let source = Path::new(&cfg.master_path).parent().unwrap().join("source");
        let media = source.join("photo.jpg");
        fs::write(&media, b"same bytes").unwrap();
        let hash = storage::sha256(&media).unwrap();
        let conn = catalog::open(&catalog_path(&cfg)).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'photo.jpg','photo','jpg',?2,'file',10,'master',?2)",params![hash,Utc::now().to_rfc3339()]).unwrap();
        drop(conn);
        let job = queue(&cfg, &source_id).unwrap();
        let first = run(&cfg, &job, &CancellationToken::default()).unwrap();
        assert_eq!(first.duplicates, 1);
        assert_eq!(fs::read(&media).unwrap(), b"same bytes");
        fs::remove_file(&media).unwrap();
        let second_job = queue(&cfg, &source_id).unwrap();
        let second = run(&cfg, &second_job, &CancellationToken::default()).unwrap();
        assert_eq!(second.missing, 1);
        let conn = catalog::open(&catalog_path(&cfg)).unwrap();
        assert_eq!(
            conn.query_row("SELECT state FROM occurrence_presence", [], |row| row
                .get::<_, String>(0))
                .unwrap(),
            "missing"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn offline_source_fails_before_a_job_is_created() {
        let (root, cfg, source_id) = fixture();
        let source = Path::new(&cfg.master_path).parent().unwrap().join("source");
        fs::remove_dir_all(&source).unwrap();
        let error = queue(&cfg, &source_id).unwrap_err();
        assert!(error.contains("offline"));
        let conn = catalog::open(&catalog_path(&cfg)).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*)FROM jobs WHERE job_kind='source_sync'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
