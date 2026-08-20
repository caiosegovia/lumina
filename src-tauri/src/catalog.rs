use rusqlite::{Connection, Result};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

static INITIALIZED: OnceLock<Mutex<HashMap<PathBuf, usize>>> = OnceLock::new();

fn connect(path: &Path) -> Result<Connection> {
    let db = Connection::open(path)?;
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;")?;
    Ok(db)
}

pub fn open(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let key = path.to_path_buf();
    let initialized = INITIALIZED.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = initialized
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if guard.contains_key(&key) {
        return connect(path);
    }
    let db = connect(path)?;
    db.execute_batch("
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
      CREATE TRIGGER IF NOT EXISTS queue_thumbnail_completion_guard BEFORE UPDATE OF state ON work_queue WHEN new.kind='thumbnail' AND new.state='completed' AND NOT EXISTS(SELECT 1 FROM thumbnails t WHERE t.asset_id=new.asset_id AND t.state='ready') BEGIN SELECT RAISE(ABORT,'thumbnail completion invariant'); END;
      CREATE TRIGGER IF NOT EXISTS queue_backup_completion_guard BEFORE UPDATE OF state ON work_queue WHEN new.kind='backup' AND new.state='completed' AND NOT EXISTS(SELECT 1 FROM backup_entries b WHERE b.asset_id=new.asset_id AND b.state='verified') BEGIN SELECT RAISE(ABORT,'backup completion invariant'); END;
      CREATE VIEW IF NOT EXISTS durable_work AS
        SELECT 'job_item:'||id AS work_id,job_id,current_stage AS kind,state,attempts,updated_at FROM job_items
        UNION ALL
        SELECT 'queue:'||id,job_id,kind,state,attempts,updated_at FROM work_queue;
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
    let work_queue_has_priority = db
        .prepare("PRAGMA table_info(work_queue)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(Result::ok)
        .any(|column| column == "priority");
    if !work_queue_has_priority {
        db.execute(
            "ALTER TABLE work_queue ADD COLUMN priority INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    for (name, definition) in [("volume_id", "TEXT"), ("mount_path", "TEXT")] {
        let exists = db
            .prepare("PRAGMA table_info(sources)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(Result::ok)
            .any(|column| column == name);
        if !exists {
            db.execute(
                &format!("ALTER TABLE sources ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    db.execute("UPDATE sources SET mount_path=path,volume_id=volume_label,path=volume_label||'::'||path WHERE volume_id IS NULL AND path NOT LIKE 'lumina://%'",[])?;
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
    db.pragma_update(None, "user_version", 11)?;
    db.execute_batch("PRAGMA optimize;")?;
    db.execute(
        "UPDATE jobs SET state='waiting_space',stage='space_check',finished_at=NULL WHERE state='failed' AND processed_items=0 AND interruption_reason LIKE 'Espaço insuficiente:%'",
        [],
    )?;
    compact_telemetry(&db)?;
    guard.insert(key, 1);
    Ok(db)
}

fn compact_telemetry(db: &Connection) -> Result<usize> {
    db.execute(
        "DELETE FROM process_events WHERE id < (SELECT COALESCE(MAX(id)-50000,0) FROM process_events) AND state='completed'",
        [],
    )
}

pub fn snapshot(source_path: &Path, destination: &Path) -> Result<()> {
    use rusqlite::backup::Backup;
    use std::time::Duration;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
    }
    let temporary = destination.with_extension("sqlite.lumina-replacement");
    let source = open(source_path)?;
    let mut target = Connection::open(&temporary)?;
    Backup::new(&source, &mut target)?.run_to_completion(128, Duration::from_millis(5), None)?;
    drop(target);
    drop(source);
    crate::storage::replace_file(&temporary, destination)
        .map_err(|_| rusqlite::Error::InvalidPath(destination.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use std::{fs, time::Instant};
    use uuid::Uuid;

    #[test]
    fn schema_initialization_runs_once_per_catalog() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let path = root.join("one.sqlite");
        drop(open(&path).unwrap());
        drop(open(&path).unwrap());
        assert_eq!(
            *INITIALIZED
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .get(&path)
                .unwrap(),
            1
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn durable_queue_rejects_completion_without_its_invariant() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let db = open(&root.join("guard.sqlite")).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('s','s','s','s')",
            [],
        )
        .unwrap();
        db.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('j','s','s','queued',?1,?1)",[&now]).unwrap();
        db.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'file',1,'a.jpg',?2)",params!["a".repeat(64),now]).unwrap();
        db.execute("INSERT INTO work_queue(job_id,asset_id,kind,state,created_at,updated_at)VALUES('j','a','thumbnail','processing',?1,?1)",[&now]).unwrap();
        assert!(db
            .execute(
                "UPDATE work_queue SET state='completed' WHERE job_id='j'",
                []
            )
            .is_err());
        db.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES('a',2,'thumb.jpg','ready',?1)",[&now]).unwrap();
        assert_eq!(
            db.execute(
                "UPDATE work_queue SET state='completed' WHERE job_id='j'",
                []
            )
            .unwrap(),
            1
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn durable_work_exposes_every_pipeline_family() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let db = open(&root.join("work.sqlite")).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('s','s','s','s')",
            [],
        )
        .unwrap();
        db.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('j','s','s','queued',?1,?1)",[&now]).unwrap();
        db.execute("INSERT INTO job_items(job_id,source_path,filename,extension,media_type,current_stage,state,created_at,updated_at)VALUES('j','p','p.jpg','jpg','photo','validation','queued',?1,?1)",[&now]).unwrap();
        for kind in ["thumbnail", "backup", "verification"] {
            db.execute("INSERT INTO work_queue(job_id,kind,state,created_at,updated_at)VALUES('j',?1,'pending',?2,?2)",params![kind,now]).unwrap();
        }
        let kinds = db
            .prepare("SELECT DISTINCT kind FROM durable_work ORDER BY kind")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            kinds,
            vec!["backup", "thumbnail", "validation", "verification"]
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn telemetry_retention_never_deletes_failures() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let db = open(&root.join("retention.sqlite")).unwrap();
        let tx = db.unchecked_transaction().unwrap();
        for index in 0..50_010 {
            tx.execute("INSERT INTO process_events(at,tool,logical_command,state,details)VALUES(datetime('now'),'tool','work','completed',?1)",[index]).unwrap();
        }
        tx.execute("INSERT INTO process_events(at,tool,logical_command,state,details)VALUES(datetime('now'),'tool','work','failed','essential failure')",[]).unwrap();
        tx.commit().unwrap();
        assert!(compact_telemetry(&db).unwrap() > 0);
        assert_eq!(db.query_row("SELECT COUNT(*) FROM process_events WHERE state='failed' AND details='essential failure'",[],|row|row.get::<_,i64>(0)).unwrap(),1);
        assert!(
            db.query_row("SELECT COUNT(*) FROM process_events", [], |row| row
                .get::<_, i64>(0))
                .unwrap()
                <= 50_001
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sqlite_backup_contains_committed_wal_data() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let source = root.join("live.sqlite");
        let destination = root.join("backup/catalog.sqlite");
        let db = open(&source).unwrap();
        db.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('snapshot','Snapshot','S:/','S')",
            [],
        )
        .unwrap();
        snapshot(&source, &destination).unwrap();
        let restored = Connection::open(destination).unwrap();
        assert_eq!(
            restored
                .query_row(
                    "SELECT COUNT(*) FROM sources WHERE id='snapshot'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        drop(restored);
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wal_readers_remain_available_during_a_write_transaction() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let path = root.join("concurrency.sqlite");
        drop(open(&path).unwrap());
        let writer_path = path.clone();
        let writer = std::thread::spawn(move || {
            let mut db = open(&writer_path).unwrap();
            let tx = db.transaction().unwrap();
            tx.execute(
                "INSERT INTO sources(id,name,path,volume_label)VALUES('writer','writer','w','w')",
                [],
            )
            .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(75));
            tx.commit().unwrap();
        });
        std::thread::sleep(std::time::Duration::from_millis(15));
        let started = Instant::now();
        let reader = open(&path).unwrap();
        reader
            .query_row("SELECT COUNT(*) FROM sources", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap();
        assert!(started.elapsed() < std::time::Duration::from_millis(50));
        writer.join().unwrap();
        drop(reader);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_handles_one_hundred_thousand_assets() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let mut db = open(&root.join("load.sqlite")).unwrap();
        assert_eq!(
            db.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
                .unwrap(),
            11
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
        let mut page_latencies = Vec::new();
        for _ in 0..20 {
            let page_started = Instant::now();
            let page = crate::gallery::search(
                &db,
                &crate::models::GalleryRequest {
                    filters: crate::models::GalleryFilters {
                        year: Some(2026),
                        media_type: Some("photo".into()),
                        ..Default::default()
                    },
                    cursor: gallery.next_cursor.clone(),
                    limit: Some(100),
                },
            )
            .unwrap();
            assert_eq!(page.assets.len(), 100);
            page_latencies.push(page_started.elapsed().as_millis());
        }
        page_latencies.sort_unstable();
        let p50 = page_latencies[page_latencies.len() / 2];
        let p95 = page_latencies[page_latencies.len() * 95 / 100];
        assert!(p95 < 300, "p95 da paginação com 100 mil itens: {p95} ms");
        #[cfg(windows)]
        let working_set = unsafe {
            use windows_sys::Win32::System::{
                ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
                Threading::GetCurrentProcess,
            };
            let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
            counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
            assert_ne!(
                K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb),
                0
            );
            counters.WorkingSetSize
        };
        #[cfg(not(windows))]
        let working_set = 0usize;
        eprintln!("BENCHMARK gallery_records=100000 page_size=100 p50_ms={p50} p95_ms={p95} working_set_bytes={working_set}");
        assert!(
            working_set == 0 || working_set < 768 * 1024 * 1024,
            "working set acima de 768 MiB: {working_set}"
        );
        assert!(
            started.elapsed().as_secs() < 5,
            "consultas principais excederam cinco segundos"
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
}
