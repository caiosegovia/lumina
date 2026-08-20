use rusqlite::{Connection, Result};
use std::{fs, path::Path};

pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let db = Connection::open(path)?;
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;
      CREATE TABLE IF NOT EXISTS sources(id TEXT PRIMARY KEY,name TEXT NOT NULL,path TEXT NOT NULL UNIQUE,volume_label TEXT NOT NULL,available INTEGER NOT NULL DEFAULT 1,last_scan TEXT,asset_count INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS assets(id TEXT PRIMARY KEY,hash TEXT NOT NULL UNIQUE,filename TEXT NOT NULL,media_type TEXT NOT NULL,extension TEXT NOT NULL,captured_at TEXT NOT NULL,date_source TEXT NOT NULL,bytes INTEGER NOT NULL,width INTEGER,height INTEGER,duration REAL,camera TEXT,latitude REAL,longitude REAL,master_path TEXT NOT NULL,protection_state TEXT NOT NULL DEFAULT 'consolidated',created_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS occurrences(id TEXT PRIMARY KEY,asset_id TEXT NOT NULL REFERENCES assets(id),source_id TEXT NOT NULL REFERENCES sources(id),path TEXT NOT NULL,seen_at TEXT NOT NULL,UNIQUE(source_id,path));
      CREATE TABLE IF NOT EXISTS jobs(id TEXT PRIMARY KEY,source_id TEXT NOT NULL REFERENCES sources(id),source_path TEXT NOT NULL,state TEXT NOT NULL,created_at TEXT NOT NULL,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS pending_files(id INTEGER PRIMARY KEY AUTOINCREMENT,job_id TEXT NOT NULL REFERENCES jobs(id),path TEXT NOT NULL,filename TEXT NOT NULL,extension TEXT NOT NULL,media_type TEXT NOT NULL,bytes INTEGER NOT NULL,modified_at TEXT NOT NULL,hash TEXT,status TEXT NOT NULL,error TEXT);
      CREATE TABLE IF NOT EXISTS events(id INTEGER PRIMARY KEY AUTOINCREMENT,job_id TEXT NOT NULL,at TEXT NOT NULL,path TEXT NOT NULL,state TEXT NOT NULL,details TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS backup_entries(asset_id TEXT PRIMARY KEY REFERENCES assets(id),path TEXT NOT NULL,hash TEXT NOT NULL,verified_at TEXT,state TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS albums(id TEXT PRIMARY KEY,name TEXT NOT NULL,created_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS album_assets(album_id TEXT NOT NULL REFERENCES albums(id),asset_id TEXT NOT NULL REFERENCES assets(id),PRIMARY KEY(album_id,asset_id));
      CREATE TABLE IF NOT EXISTS tags(id TEXT PRIMARY KEY,name TEXT NOT NULL UNIQUE);
      CREATE TABLE IF NOT EXISTS asset_tags(asset_id TEXT NOT NULL REFERENCES assets(id),tag_id TEXT NOT NULL REFERENCES tags(id),PRIMARY KEY(asset_id,tag_id));")?;
    for (name, definition) in [
        ("stage", "TEXT NOT NULL DEFAULT 'discovery'"),
        ("current_file", "TEXT"),
        ("processed_items", "INTEGER NOT NULL DEFAULT 0"),
        ("total_items", "INTEGER NOT NULL DEFAULT 0"),
        ("processed_bytes", "INTEGER NOT NULL DEFAULT 0"),
        ("total_bytes", "INTEGER NOT NULL DEFAULT 0"),
        ("imported_count", "INTEGER NOT NULL DEFAULT 0"),
        ("duplicate_count", "INTEGER NOT NULL DEFAULT 0"),
        ("excluded_count", "INTEGER NOT NULL DEFAULT 0"),
        ("failed_count", "INTEGER NOT NULL DEFAULT 0"),
        ("started_at", "TEXT"),
        ("finished_at", "TEXT"),
    ] {
        let exists = db
            .prepare("PRAGMA table_info(jobs)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|column| column == name);
        if !exists {
            db.execute(
                &format!("ALTER TABLE jobs ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    db.execute_batch("BEGIN IMMEDIATE;
      CREATE TABLE IF NOT EXISTS schema_migrations(version INTEGER PRIMARY KEY,applied_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS job_items(
        id INTEGER PRIMARY KEY AUTOINCREMENT,job_id TEXT NOT NULL REFERENCES jobs(id),source_path TEXT NOT NULL,filename TEXT NOT NULL,extension TEXT NOT NULL,media_type TEXT NOT NULL,
        bytes INTEGER NOT NULL DEFAULT 0,modified_at TEXT,sha256 TEXT,destination_path TEXT,temp_path TEXT,current_stage TEXT NOT NULL DEFAULT 'discovery',state TEXT NOT NULL DEFAULT 'queued',
        validation_state TEXT,attempts INTEGER NOT NULL DEFAULT 0,last_error_kind TEXT,last_error TEXT,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,UNIQUE(job_id,source_path));
      CREATE INDEX IF NOT EXISTS idx_job_items_work ON job_items(job_id,state,current_stage);
      CREATE UNIQUE INDEX IF NOT EXISTS idx_pending_job_path ON pending_files(job_id,path);
      CREATE INDEX IF NOT EXISTS idx_assets_captured_at ON assets(captured_at DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_filename ON assets(filename);
      CREATE INDEX IF NOT EXISTS idx_assets_media_type ON assets(media_type);
      CREATE INDEX IF NOT EXISTS idx_assets_timeline ON assets(captured_at DESC,id DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_type_timeline ON assets(media_type,captured_at DESC,id DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_protection_timeline ON assets(protection_state,captured_at DESC,id DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_extension_timeline ON assets(extension,captured_at DESC,id DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_camera_timeline ON assets(camera,captured_at DESC,id DESC);
      CREATE INDEX IF NOT EXISTS idx_assets_bytes_hash ON assets(bytes,hash);
      CREATE INDEX IF NOT EXISTS idx_assets_year_type ON assets(substr(captured_at,1,4),media_type);
      CREATE INDEX IF NOT EXISTS idx_assets_pending_protection ON assets(protection_state,bytes) WHERE protection_state!='replica_verified';
      CREATE INDEX IF NOT EXISTS idx_occurrences_asset ON occurrences(asset_id);
      CREATE INDEX IF NOT EXISTS idx_occurrences_source_asset ON occurrences(source_id,asset_id);
      CREATE INDEX IF NOT EXISTS idx_asset_tags_tag_asset ON asset_tags(tag_id,asset_id);
      CREATE INDEX IF NOT EXISTS idx_album_assets_album_asset ON album_assets(album_id,asset_id);
      CREATE VIRTUAL TABLE IF NOT EXISTS assets_fts USING fts5(asset_id UNINDEXED,filename,camera,tokenize='trigram');
      INSERT INTO assets_fts(asset_id,filename,camera) SELECT a.id,a.filename,COALESCE(a.camera,'') FROM assets a WHERE NOT EXISTS(SELECT 1 FROM assets_fts LIMIT 1);
      CREATE TRIGGER IF NOT EXISTS assets_fts_insert AFTER INSERT ON assets BEGIN INSERT INTO assets_fts(asset_id,filename,camera)VALUES(new.id,new.filename,COALESCE(new.camera,'')); END;
      CREATE TRIGGER IF NOT EXISTS assets_fts_update AFTER UPDATE OF filename,camera ON assets BEGIN DELETE FROM assets_fts WHERE asset_id=old.id; INSERT INTO assets_fts(asset_id,filename,camera)VALUES(new.id,new.filename,COALESCE(new.camera,'')); END;
      CREATE TRIGGER IF NOT EXISTS assets_fts_delete AFTER DELETE ON assets BEGIN DELETE FROM assets_fts WHERE asset_id=old.id; END;
      CREATE TABLE IF NOT EXISTS job_counters(job_id TEXT PRIMARY KEY REFERENCES jobs(id),imported INTEGER NOT NULL DEFAULT 0,duplicates INTEGER NOT NULL DEFAULT 0,excluded INTEGER NOT NULL DEFAULT 0,failed INTEGER NOT NULL DEFAULT 0,validated INTEGER NOT NULL DEFAULT 0,thumbnailed INTEGER NOT NULL DEFAULT 0,backed_up INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS media_validation(job_item_id INTEGER PRIMARY KEY REFERENCES job_items(id),state TEXT NOT NULL,tool TEXT NOT NULL,details TEXT,checked_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS thumbnails(asset_id TEXT PRIMARY KEY REFERENCES assets(id),generator_version INTEGER NOT NULL,path TEXT NOT NULL,width INTEGER,height INTEGER,state TEXT NOT NULL,last_error TEXT,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS process_events(id INTEGER PRIMARY KEY AUTOINCREMENT,job_id TEXT,job_item_id INTEGER,at TEXT NOT NULL,tool TEXT NOT NULL,logical_command TEXT NOT NULL,duration_ms INTEGER,exit_code INTEGER,state TEXT NOT NULL,error_kind TEXT,details TEXT);
      CREATE TABLE IF NOT EXISTS library_lock(id INTEGER PRIMARY KEY CHECK(id=1),instance_id TEXT NOT NULL,process_id INTEGER NOT NULL,heartbeat_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS asset_edits(id INTEGER PRIMARY KEY AUTOINCREMENT,asset_id TEXT NOT NULL REFERENCES assets(id),field TEXT NOT NULL,old_value TEXT,new_value TEXT NOT NULL,edited_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS job_metrics(job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,stage TEXT NOT NULL,duration_ms INTEGER NOT NULL,items INTEGER NOT NULL DEFAULT 0,bytes INTEGER NOT NULL DEFAULT 0,recorded_at TEXT NOT NULL,PRIMARY KEY(job_id,stage));
      CREATE TABLE IF NOT EXISTS job_selection(job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,job_item_id INTEGER NOT NULL REFERENCES job_items(id) ON DELETE CASCADE,selected INTEGER NOT NULL DEFAULT 1,batch_no INTEGER NOT NULL DEFAULT 1,PRIMARY KEY(job_id,job_item_id));
      CREATE INDEX IF NOT EXISTS idx_job_selection_batch ON job_selection(job_id,selected,batch_no);
      CREATE TABLE IF NOT EXISTS work_queue(id INTEGER PRIMARY KEY AUTOINCREMENT,job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,asset_id TEXT REFERENCES assets(id),job_item_id INTEGER REFERENCES job_items(id),kind TEXT NOT NULL,state TEXT NOT NULL DEFAULT 'pending',attempts INTEGER NOT NULL DEFAULT 0,last_error TEXT,created_at TEXT NOT NULL,updated_at TEXT NOT NULL,UNIQUE(job_id,asset_id,kind));
      CREATE INDEX IF NOT EXISTS idx_work_queue_next ON work_queue(kind,state,id);
      CREATE TABLE IF NOT EXISTS library_migrations(id TEXT PRIMARY KEY,old_master TEXT NOT NULL,new_master TEXT NOT NULL,state TEXT NOT NULL,processed_items INTEGER NOT NULL DEFAULT 0,total_items INTEGER NOT NULL DEFAULT 0,processed_bytes INTEGER NOT NULL DEFAULT 0,total_bytes INTEGER NOT NULL DEFAULT 0,last_error TEXT,created_at TEXT NOT NULL,updated_at TEXT NOT NULL);
      CREATE TABLE IF NOT EXISTS library_rollups(dimension TEXT NOT NULL,key TEXT NOT NULL,items INTEGER NOT NULL DEFAULT 0,bytes INTEGER NOT NULL DEFAULT 0,PRIMARY KEY(dimension,key));
      CREATE TRIGGER IF NOT EXISTS rollup_asset_insert AFTER INSERT ON assets BEGIN
        INSERT INTO library_rollups(dimension,key,items,bytes)VALUES('type',new.media_type,1,new.bytes)ON CONFLICT(dimension,key)DO UPDATE SET items=items+1,bytes=bytes+new.bytes;
        INSERT INTO library_rollups(dimension,key,items,bytes)VALUES('year',substr(new.captured_at,1,4),1,new.bytes)ON CONFLICT(dimension,key)DO UPDATE SET items=items+1,bytes=bytes+new.bytes;
        INSERT INTO library_rollups(dimension,key,items,bytes)VALUES('protection',new.protection_state,1,new.bytes)ON CONFLICT(dimension,key)DO UPDATE SET items=items+1,bytes=bytes+new.bytes;
      END;
      CREATE TRIGGER IF NOT EXISTS rollup_asset_delete AFTER DELETE ON assets BEGIN
        UPDATE library_rollups SET items=items-1,bytes=bytes-old.bytes WHERE dimension='type' AND key=old.media_type;
        UPDATE library_rollups SET items=items-1,bytes=bytes-old.bytes WHERE dimension='year' AND key=substr(old.captured_at,1,4);
        UPDATE library_rollups SET items=items-1,bytes=bytes-old.bytes WHERE dimension='protection' AND key=old.protection_state;
      END;
      CREATE TRIGGER IF NOT EXISTS rollup_asset_protection AFTER UPDATE OF protection_state ON assets WHEN old.protection_state!=new.protection_state BEGIN
        UPDATE library_rollups SET items=items-1,bytes=bytes-old.bytes WHERE dimension='protection' AND key=old.protection_state;
        INSERT INTO library_rollups(dimension,key,items,bytes)VALUES('protection',new.protection_state,1,new.bytes)ON CONFLICT(dimension,key)DO UPDATE SET items=items+1,bytes=bytes+new.bytes;
      END;
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(1,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(2,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(3,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(4,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(5,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(6,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(7,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(8,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(9,datetime('now'));
      INSERT OR IGNORE INTO schema_migrations(version,applied_at)VALUES(10,datetime('now'));
      COMMIT;")?;
    let rollups: i64 = db.query_row("SELECT COUNT(*) FROM library_rollups", [], |r| r.get(0))?;
    if rollups == 0 {
        db.execute_batch("INSERT INTO library_rollups(dimension,key,items,bytes)SELECT 'type',media_type,COUNT(*),SUM(bytes)FROM assets GROUP BY media_type;INSERT INTO library_rollups(dimension,key,items,bytes)SELECT 'year',substr(captured_at,1,4),COUNT(*),SUM(bytes)FROM assets GROUP BY 2;INSERT INTO library_rollups(dimension,key,items,bytes)SELECT 'protection',protection_state,COUNT(*),SUM(bytes)FROM assets GROUP BY protection_state;")?;
    }
    for (name, definition) in [
        ("interruption_reason", "TEXT"),
        ("stage_processed_items", "INTEGER NOT NULL DEFAULT 0"),
        ("stage_total_items", "INTEGER NOT NULL DEFAULT 0"),
        ("stage_processed_bytes", "INTEGER NOT NULL DEFAULT 0"),
        ("stage_total_bytes", "INTEGER NOT NULL DEFAULT 0"),
        ("bytes_per_second", "REAL"),
        ("estimated_seconds_remaining", "INTEGER"),
        ("library_state", "TEXT NOT NULL DEFAULT 'pending'"),
        ("backup_state", "TEXT NOT NULL DEFAULT 'pending'"),
        ("instance_id", "TEXT"),
    ] {
        let exists = db
            .prepare("PRAGMA table_info(jobs)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|column| column == name);
        if !exists {
            db.execute(
                &format!("ALTER TABLE jobs ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    for (name, definition) in [
        ("captured_at", "TEXT"),
        ("date_source", "TEXT"),
        ("width", "INTEGER"),
        ("height", "INTEGER"),
        ("duration", "REAL"),
        ("camera", "TEXT"),
        ("latitude", "REAL"),
        ("longitude", "REAL"),
    ] {
        let exists = db
            .prepare("PRAGMA table_info(job_items)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|column| column == name);
        if !exists {
            db.execute(
                &format!("ALTER TABLE job_items ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    db.pragma_update(None, "user_version", 10)?;
    db.execute_batch("PRAGMA optimize;")?;
    db.execute(
        "UPDATE jobs SET state='waiting_space',stage='space_check',finished_at=NULL WHERE state='failed' AND processed_items=0 AND interruption_reason LIKE 'Espaço insuficiente:%'",
        [],
    )?;
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use std::{fs, time::Instant};
    use uuid::Uuid;

    #[test]
    fn catalog_handles_one_hundred_thousand_assets() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let mut db = open(&root.join("load.sqlite")).unwrap();
        assert_eq!(
            db.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            10
        );
        assert_eq!(
            db.query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            10
        );
        let transaction = db.transaction().unwrap();
        {
            let mut insert = transaction.prepare("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','jpg',?4,'file',100,?5,?4)").unwrap();
            for index in 0..100_000u32 {
                let value = format!("{index:064x}");
                insert
                    .execute(params![
                        format!("id-{index}"),
                        value,
                        format!("IMG_{index:06}.jpg"),
                        format!("2026-01-{:02}T00:00:00Z", index % 28 + 1),
                        format!("master/{index}.jpg")
                    ])
                    .unwrap();
            }
        }
        transaction.commit().unwrap();
        let started = Instant::now();
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM assets WHERE media_type='photo'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let newest: String = db
            .query_row(
                "SELECT filename FROM assets ORDER BY captured_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 100_000);
        assert_eq!(
            db.query_row(
                "SELECT items FROM library_rollups WHERE dimension='type' AND key='photo'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            100_000
        );
        assert!(newest.starts_with("IMG_"));
        let gallery = crate::gallery::search(
            &db,
            &crate::models::GalleryRequest {
                filters: crate::models::GalleryFilters {
                    year: Some(2026),
                    media_type: Some("photo".into()),
                    ..Default::default()
                },
                cursor: None,
                limit: Some(100),
            },
        )
        .unwrap();
        assert_eq!(gallery.matched, 100_000);
        assert_eq!(gallery.assets.len(), 100);
        assert!(gallery.next_cursor.is_some());
        assert!(
            started.elapsed().as_secs() < 5,
            "consultas principais excederam cinco segundos"
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
}
