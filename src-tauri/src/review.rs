use crate::{catalog, models::*};
use rusqlite::Connection;
use std::path::Path;

pub fn summary(cfg: &LibraryConfig) -> Result<ReviewSummary, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    summary_from(&conn)
}

fn summary_from(conn: &Connection) -> Result<ReviewSummary, String> {
    conn.query_row(
        "SELECT
          COALESCE(SUM(COALESCE(us.review_later,0)=1),0),
          COALESCE(SUM(a.date_source IN('file','fallback','filesystem') OR CAST(substr(a.captured_at,1,4) AS INTEGER)<1990 OR CAST(substr(a.captured_at,1,4) AS INTEGER)>CAST(strftime('%Y','now') AS INTEGER)+1),0),
          COALESCE(SUM(COALESCE(t.state,'missing')!='ready'),0),
          COALESCE(SUM(tm.asset_id IS NULL OR tm.inventory_state!='complete'),0),
          COALESCE(SUM(a.protection_state!='replica_verified'),0),
          COALESCE(SUM((SELECT COUNT(*) FROM active_occurrences o WHERE o.asset_id=a.id)>1 AND NOT EXISTS(SELECT 1 FROM duplicate_decisions d WHERE d.asset_id=a.id)),0)
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
}
