use crate::{catalog, models::*};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub fn summary(cfg: &LibraryConfig) -> Result<ReviewSummary, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    summary_from(&conn)
}

pub fn undo_last(cfg: &LibraryConfig) -> Result<BatchResult, String> {
    let mut conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let edit:Option<(i64,String,String,String)>=conn.query_row("SELECT e.id,e.asset_id,e.field,e.old_value FROM asset_edits e WHERE NOT EXISTS(SELECT 1 FROM undone_asset_edits u WHERE u.edit_id=e.id)ORDER BY e.id DESC LIMIT 1",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional().map_err(|error|error.to_string())?;
    let Some((id, asset, field, old)) = edit else {
        return Ok(BatchResult { affected: 0 });
    };
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    match field.as_str() {
        "captured_at" => {
            tx.execute(
                "UPDATE assets SET captured_at=?2,date_source='user_corrected' WHERE id=?1",
                params![asset, old],
            )
            .map_err(|error| error.to_string())?;
        }
        "user_state" => {
            let value: (bool, i64, bool, String) =
                serde_json::from_str(&old).map_err(|error| error.to_string())?;
            tx.execute("INSERT INTO asset_user_state(asset_id,favorite,rating,review_later,description,updated_at)VALUES(?1,?2,?3,?4,?5,?6)ON CONFLICT(asset_id)DO UPDATE SET favorite=excluded.favorite,rating=excluded.rating,review_later=excluded.review_later,description=excluded.description,updated_at=excluded.updated_at",params![asset,value.0,value.1,value.2,value.3,Utc::now().to_rfc3339()]).map_err(|error|error.to_string())?;
        }
        _ => return Err("A última alteração ainda não possui reversão automática".into()),
    }
    tx.execute(
        "INSERT INTO undone_asset_edits(edit_id,undone_at)VALUES(?1,?2)",
        params![id, Utc::now().to_rfc3339()],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())?;
    Ok(BatchResult { affected: 1 })
}

fn summary_from(conn: &Connection) -> Result<ReviewSummary, String> {
    conn.query_row(
        "SELECT
          COALESCE(SUM(COALESCE(us.review_later,0)=1),0),
          COALESCE(SUM(a.date_source IN('file','fallback','filesystem') OR CAST(substr(a.captured_at,1,4) AS INTEGER)<1990 OR CAST(substr(a.captured_at,1,4) AS INTEGER)>CAST(strftime('%Y','now') AS INTEGER)+1),0),
          COALESCE(SUM(COALESCE(t.state,'missing')!='ready'),0),
          COALESCE(SUM(tm.asset_id IS NULL OR tm.inventory_state!='complete'),0),
          COALESCE(SUM(a.protection_state!='replica_verified'),0),
          COALESCE(SUM((SELECT COUNT(*) FROM active_occurrences o WHERE o.asset_id=a.id)>1 AND NOT EXISTS(SELECT 1 FROM duplicate_decisions d WHERE d.asset_id=a.id)),0),
          COALESCE(SUM(COALESCE(tm.inventory_state,'missing')='failed' OR tm.inventory_error IS NOT NULL OR COALESCE(t.state,'missing')='failed'),0)
         FROM assets a
         LEFT JOIN asset_user_state us ON us.asset_id=a.id
         LEFT JOIN thumbnails t ON t.asset_id=a.id
         LEFT JOIN asset_technical_metadata tm ON tm.asset_id=a.id",
        [],
        |row| {
            Ok(ReviewSummary {
                review_later: row.get(0)?,
                suspicious_dates: row.get(1)?,
                missing_previews: row.get(2)?,
                incomplete_metadata: row.get(3)?,
                pending_protection: row.get(4)?,
                undecided_duplicates: row.get(5)?,
                technical_failures: row.get(6)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use rusqlite::params;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn summary_explains_every_review_queue_without_double_count_assumptions() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let conn = catalog::open(&root.join("review.sqlite")).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,protection_state,created_at)VALUES('a',?1,'a.jpg','photo','jpg','1980-01-01T00:00:00Z','file',1,'a','source_only',?2)",params!["a".repeat(64),now]).unwrap();
        conn.execute(
            "INSERT INTO asset_user_state(asset_id,review_later,updated_at)VALUES('a',1,?1)",
            [now],
        )
        .unwrap();
        let result = summary_from(&conn).unwrap();
        assert_eq!(result.review_later, 1);
        assert_eq!(result.suspicious_dates, 1);
        assert_eq!(result.missing_previews, 1);
        assert_eq!(result.incomplete_metadata, 1);
        assert_eq!(result.pending_protection, 1);
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn undo_restores_the_latest_catalog_edit_once() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(root.join(".lumina")).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: root.to_string_lossy().into(),
            backup_path: root.join("backup").to_string_lossy().into(),
            created_at: now.clone(),
        };
        let conn = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,protection_state,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'user_corrected',1,'a','source_only',?3)",params!["a".repeat(64),"2025-01-01T00:00:00Z",now]).unwrap();
        conn.execute("INSERT INTO asset_edits(asset_id,field,old_value,new_value,edited_at)VALUES('a','captured_at','2020-01-01T00:00:00Z','2025-01-01T00:00:00Z',?1)",[chrono::Utc::now().to_rfc3339()]).unwrap();
        drop(conn);
        assert_eq!(undo_last(&cfg).unwrap().affected, 1);
        assert_eq!(undo_last(&cfg).unwrap().affected, 0);
        let conn = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        let restored: String = conn
            .query_row("SELECT captured_at FROM assets WHERE id='a'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(restored, "2020-01-01T00:00:00Z");
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
