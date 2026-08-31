use crate::{
    catalog,
    models::{ImportEvent, JobEventPage, LibraryConfig, ReportExport},
    storage,
};
use rusqlite::{params, OptionalExtension};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn page(
    cfg: &LibraryConfig,
    job: &str,
    cursor: i64,
    filter: &str,
) -> Result<JobEventPage, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let state = match filter {
        "errors" => "failed",
        "duplicates" => "duplicate",
        "excluded" => "excluded",
        "completed" => "completed",
        _ => "",
    };
    let mut stmt=conn.prepare("SELECT id,job_id,at,path,state,details FROM events WHERE job_id=?1 AND id>?2 AND (?3='' OR state=?3) ORDER BY id LIMIT 200").map_err(|e|e.to_string())?;
    let events = stmt
        .query_map(params![job, cursor, state], |r| {
            Ok(ImportEvent {
                id: r.get(0)?,
                job_id: r.get(1)?,
                at: r.get(2)?,
                path: r.get(3)?,
                state: r.get(4)?,
                details: r.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let next_cursor = events.last().map(|e| e.id).unwrap_or(cursor);
    Ok(JobEventPage {
        events,
        next_cursor,
    })
}

fn csv(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
pub fn export(cfg: &LibraryConfig, job: &str, format: &str) -> Result<ReportExport, String> {
    if !matches!(format, "jsonl" | "csv") {
        return Err("Formato de relatório inválido".into());
    }
    let mut events = Vec::new();
    let mut cursor = 0;
    loop {
        let batch = page(cfg, job, cursor, "")?;
        if batch.events.is_empty() {
            break;
        }
        cursor = batch.next_cursor;
        events.extend(batch.events);
    }
    let dir = Path::new(&cfg.master_path).join(".lumina/reports");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("import-{job}.{format}"));
    let mut output = File::create(&path).map_err(|e| e.to_string())?;
    if format == "csv" {
        writeln!(output, "id,job_id,at,path,state,details").map_err(|e| e.to_string())?
    }
    for event in &events {
        if format == "jsonl" {
            writeln!(
                output,
                "{}",
                serde_json::to_string(event).map_err(|e| e.to_string())?
            )
            .map_err(|e| e.to_string())?
        } else {
            writeln!(
                output,
                "{},{},{},{},{},{}",
                event.id,
                csv(&event.job_id),
                csv(&event.at),
                csv(&event.path),
                csv(&event.state),
                csv(&event.details)
            )
            .map_err(|e| e.to_string())?
        }
    }
    Ok(ReportExport {
        path: path.to_string_lossy().into_owned(),
        rows: events.len() as i64,
    })
}

fn grouped(conn: &rusqlite::Connection, sql: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(serde_json::json!({
                "category": row.get::<_, String>(0)?,
                "state": row.get::<_, String>(1)?,
                "items": row.get::<_, i64>(2)?
            }))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn export_diagnostics(cfg: &LibraryConfig) -> Result<ReportExport, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let integrity = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;
    let (assets, bytes, sources, jobs, failures): (i64, i64, i64, i64, i64) = conn
        .query_row(
            "SELECT (SELECT COUNT(*) FROM assets),(SELECT COALESCE(SUM(bytes),0) FROM assets),(SELECT COUNT(*) FROM sources WHERE path NOT LIKE 'lumina://%'),(SELECT COUNT(*) FROM jobs WHERE source_path NOT LIKE 'lumina://%'),(SELECT COUNT(*) FROM process_events WHERE state!='completed' OR COALESCE(exit_code,0)!=0)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|error| error.to_string())?;
    let latest_dashboard = conn.query_row(
        "SELECT generated_at,mode,total_ms,catalog_ms,rollups_ms,storage_ms,insights_ms,items FROM dashboard_metrics ORDER BY id DESC LIMIT 1",[],
        |row| Ok(serde_json::json!({"generatedAt":row.get::<_,String>(0)?,"mode":row.get::<_,String>(1)?,"totalMs":row.get::<_,i64>(2)?,"catalogMs":row.get::<_,i64>(3)?,"rollupsMs":row.get::<_,i64>(4)?,"storageMs":row.get::<_,i64>(5)?,"insightsMs":row.get::<_,i64>(6)?,"items":row.get::<_,i64>(7)?}))
    ).optional().map_err(|error|error.to_string())?;
    let document = serde_json::json!({
        "schemaVersion":1,
        "generatedAt":chrono::Utc::now().to_rfc3339(),
        "application":{"name":"Lumina","version":env!("CARGO_PKG_VERSION"),"platform":std::env::consts::OS,"architecture":std::env::consts::ARCH},
        "privacy":{"containsPaths":false,"containsFilenames":false,"containsCoordinates":false,"containsHashes":false},
        "catalog":{"integrity":integrity,"assets":assets,"bytes":bytes,"sources":sources,"jobs":jobs,"processFailures":failures},
        "media":grouped(&conn,"SELECT media_type,protection_state,COUNT(*) FROM assets GROUP BY media_type,protection_state ORDER BY 1,2")?,
        "work":grouped(&conn,"SELECT kind,state,COUNT(*) FROM work_queue GROUP BY kind,state ORDER BY 1,2")?,
        "validation":grouped(&conn,"SELECT tool,state,COUNT(*) FROM media_validation GROUP BY tool,state ORDER BY 1,2")?,
        "technicalInventory":grouped(&conn,"SELECT support_level,inventory_state,COUNT(*) FROM asset_technical_metadata GROUP BY support_level,inventory_state ORDER BY 1,2")?,
        "thumbnails":grouped(&conn,"SELECT 'thumbnail',state,COUNT(*) FROM thumbnails GROUP BY state ORDER BY state")?,
        "dashboard":latest_dashboard
    });
    let bytes = serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?;
    let dir = Path::new(&cfg.master_path).join(".lumina/reports");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let path = dir.join(format!(
        "lumina-diagnostics-{}.json",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    storage::atomic_write(&path, &bytes).map_err(|error| error.to_string())?;
    Ok(ReportExport {
        path: path.to_string_lossy().into_owned(),
        rows: 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::params;
    use uuid::Uuid;
    #[test]
    fn escapes_csv() {
        assert_eq!(csv("a\"b"), "\"a\"\"b\"")
    }
    #[test]
    fn export_is_not_truncated_at_page_size() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(master.join(".lumina")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('s','s','source','v')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('j','s','source','completed',?1,?1)",[Utc::now().to_rfc3339()]).unwrap();
        let transaction = conn.unchecked_transaction().unwrap();
        for index in 0..450 {
            transaction.execute("INSERT INTO events(job_id,at,path,state,details)VALUES('j',?1,?2,'completed','ok')",params![Utc::now().to_rfc3339(),format!("file-{index}")]).unwrap();
        }
        transaction.commit().unwrap();
        drop(conn);
        let report = export(&cfg, "j", "jsonl").unwrap();
        assert_eq!(report.rows, 450);
        assert_eq!(
            fs::read_to_string(report.path).unwrap().lines().count(),
            450
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn diagnostics_export_only_aggregates_and_omits_personal_data() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(master.join(".lumina")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "private-library-id".into(),
            name: "Segredo da família".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute("INSERT INTO sources(id,name,path,volume_label)VALUES('s','Cartão secreto','X:/Pessoa/DCIM','PRIVADO')",[]).unwrap();
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('j','s','X:/Pessoa/DCIM','completed',?1,?1)",[&now]).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,latitude,longitude,master_path,created_at)VALUES('a',?1,'nome-secreto.jpg','photo','jpg',?2,'exif',42,-23.0,-46.0,'D:/Pessoa/nome-secreto.jpg',?2)",params!["a".repeat(64),&now]).unwrap();
        drop(conn);

        let report = export_diagnostics(&cfg).unwrap();
        let contents = fs::read_to_string(report.path).unwrap();
        assert!(contents.contains("\"assets\": 1"));
        for secret in [
            "Pessoa",
            "nome-secreto",
            "Cartão secreto",
            "PRIVADO",
            "private-library-id",
            "-23.0",
            "-46.0",
        ] {
            assert!(!contents.contains(secret), "dado pessoal vazou: {secret}");
        }
        fs::remove_dir_all(root).unwrap();
    }
}
