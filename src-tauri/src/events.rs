use crate::{
    catalog,
    models::{ImportEvent, JobEventPage, LibraryConfig, ReportExport},
};
use rusqlite::params;
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
}
