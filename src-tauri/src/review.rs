use crate::{catalog, models::*};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

const FAILURE_PREDICATE: &str = "(COALESCE(tm.inventory_state,'missing')='failed' OR tm.inventory_error IS NOT NULL OR COALESCE(t.state,'missing')='failed')";

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalFailure {
    asset_id: String,
    filename: String,
    path: String,
    preview_error: Option<String>,
    metadata_error: Option<String>,
    preview_updated_at: Option<String>,
    metadata_updated_at: Option<String>,
    retryable_preview: bool,
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailurePage {
    items: Vec<TechnicalFailure>,
    total: i64,
    next_offset: Option<i64>,
}

pub fn failures(
    cfg: &LibraryConfig,
    offset: i64,
    stage: &str,
    query: &str,
) -> Result<FailurePage, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    failures_filtered_from(&conn, offset, stage, query)
}
#[cfg(test)]
fn failures_from(conn: &Connection, offset: i64) -> Result<FailurePage, String> {
    failures_filtered_from(conn, offset, "", "")
}
fn failures_filtered_from(
    conn: &Connection,
    offset: i64,
    stage: &str,
    query: &str,
) -> Result<FailurePage, String> {
    let offset = offset.max(0);
    let stage = match stage {
        "preview" | "metadata" => stage,
        _ => "",
    };
    let query = if query.trim().is_empty() {
        String::new()
    } else {
        format!("%{}%", query.trim().to_lowercase())
    };
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let from="FROM assets a LEFT JOIN thumbnails t ON t.asset_id=a.id LEFT JOIN asset_technical_metadata tm ON tm.asset_id=a.id";
    let filtered = format!("{FAILURE_PREDICATE} AND (?1='' OR (?1='preview' AND COALESCE(t.state,'')='failed') OR (?1='metadata' AND (COALESCE(tm.inventory_state,'')='failed' OR tm.inventory_error IS NOT NULL))) AND (?2='' OR LOWER(a.filename) LIKE ?2 OR LOWER(a.master_path) LIKE ?2)");
    let total = tx
        .query_row(
            &format!("SELECT COUNT(*) {from} WHERE {filtered}"),
            params![stage, query],
            |r| r.get::<_, i64>(0),
        )
        .map_err(|e| e.to_string())?;
    let mut stmt=tx.prepare(&format!("SELECT a.id,a.filename,a.master_path,CASE WHEN t.state='failed' THEN COALESCE(t.last_error,'Motivo não registrado') END,CASE WHEN tm.inventory_state='failed' OR tm.inventory_error IS NOT NULL THEN COALESCE(tm.inventory_error,'Motivo não registrado') END,t.updated_at,tm.enriched_at {from} WHERE {filtered} ORDER BY a.id LIMIT 50 OFFSET ?3")).map_err(|e|e.to_string())?;
    let items = stmt
        .query_map(params![stage, query, offset], |r| {
            let preview_error: Option<String> = r.get(3)?;
            let retryable_preview = preview_error.as_ref().is_some_and(|error| {
                let value = error.to_lowercase();
                value.contains("timeout")
                    || value.contains("tempo limite")
                    || value.contains("busy")
                    || value.contains("tempor")
            });
            Ok(TechnicalFailure {
                asset_id: r.get(0)?,
                filename: r.get(1)?,
                path: r.get(2)?,
                preview_error,
                metadata_error: r.get(4)?,
                preview_updated_at: r.get(5)?,
                metadata_updated_at: r.get(6)?,
                retryable_preview,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let next_offset =
        (offset + (items.len() as i64) < total).then_some(offset + items.len() as i64);
    Ok(FailurePage {
        items,
        total,
        next_offset,
    })
}

pub fn summary(cfg: &LibraryConfig) -> Result<ReviewSummary, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    summary_from(&conn)
}

pub fn undo_last(cfg: &LibraryConfig) -> Result<BatchResult, String> {
    let mut conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|error| error.to_string())?;
    let action: Option<(String, String, String)> = conn
        .query_row(
            "SELECT id,kind,undo_payload FROM catalog_actions WHERE state='applied' AND undo_payload IS NOT NULL ORDER BY created_at DESC,rowid DESC LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    if let Some((action_id, kind, raw)) = action {
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|error| error.to_string())?;
        let tx = conn.transaction().map_err(|error| error.to_string())?;
        let mut affected = 0i64;
        match kind.as_str() {
            "user_state_batch" => {
                for item in value["items"].as_array().into_iter().flatten() {
                    let Some(asset) = item["assetId"].as_str() else {
                        continue;
                    };
                    affected += tx.execute("INSERT INTO asset_user_state(asset_id,favorite,rating,review_later,description,updated_at)VALUES(?1,?2,?3,?4,?5,?6)ON CONFLICT(asset_id)DO UPDATE SET favorite=excluded.favorite,rating=excluded.rating,review_later=excluded.review_later,description=excluded.description,updated_at=excluded.updated_at",params![asset,item["favorite"].as_bool().unwrap_or(false),item["rating"].as_i64().unwrap_or(0),item["reviewLater"].as_bool().unwrap_or(false),item["description"].as_str().unwrap_or(""),Utc::now().to_rfc3339()]).map_err(|error|error.to_string())? as i64;
                }
            }
            "capture_date_batch" => {
                for item in value["items"].as_array().into_iter().flatten() {
                    let (Some(asset), Some(captured_at), Some(date_source)) = (
                        item["assetId"].as_str(),
                        item["capturedAt"].as_str(),
                        item["dateSource"].as_str(),
                    ) else {
                        continue;
                    };
                    affected += tx
                        .execute(
                            "UPDATE assets SET captured_at=?2,date_source=?3 WHERE id=?1",
                            params![asset, captured_at, date_source],
                        )
                        .map_err(|error| error.to_string())? as i64;
                }
            }
            "location_name_batch" => {
                for item in value["items"].as_array().into_iter().flatten() {
                    let Some(asset) = item["assetId"].as_str() else {
                        continue;
                    };
                    if let Some(name) = item["displayName"].as_str() {
                        affected += tx.execute("INSERT INTO asset_location_overrides(asset_id,display_name,updated_at)VALUES(?1,?2,?3)ON CONFLICT(asset_id)DO UPDATE SET display_name=excluded.display_name,updated_at=excluded.updated_at",params![asset,name,Utc::now().to_rfc3339()]).map_err(|error|error.to_string())? as i64;
                    } else {
                        affected +=
                            tx.execute(
                                "DELETE FROM asset_location_overrides WHERE asset_id=?1",
                                [asset],
                            )
                            .map_err(|error| error.to_string())? as i64;
                    }
                }
            }
            "tag_batch" | "album_batch" => {
                let key = if kind == "tag_batch" {
                    "tagId"
                } else {
                    "albumId"
                };
                let relation = if kind == "tag_batch" {
                    "asset_tags"
                } else {
                    "album_assets"
                };
                let foreign = if kind == "tag_batch" {
                    "tag_id"
                } else {
                    "album_id"
                };
                let Some(parent) = value[key].as_str() else {
                    return Err("Ação sem referência para reversão".into());
                };
                let sql = format!("DELETE FROM {relation} WHERE {foreign}=?1 AND asset_id=?2");
                for asset in value["assetIds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|item| item.as_str())
                {
                    affected += tx
                        .execute(&sql, params![parent, asset])
                        .map_err(|error| error.to_string())? as i64;
                }
            }
            _ => return Err("A última ação não possui reversão automática".into()),
        }
        tx.execute(
            "UPDATE catalog_actions SET state='undone',undone_at=?2 WHERE id=?1",
            params![action_id, Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())?;
        tx.commit().map_err(|error| error.to_string())?;
        return Ok(BatchResult { affected });
    }
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
        &format!("SELECT
          COALESCE(SUM(COALESCE(us.review_later,0)=1),0),
          COALESCE(SUM(a.date_source IN('file','fallback','filesystem') OR CAST(substr(a.captured_at,1,4) AS INTEGER)<1990 OR CAST(substr(a.captured_at,1,4) AS INTEGER)>CAST(strftime('%Y','now') AS INTEGER)+1),0),
          COALESCE(SUM(COALESCE(t.state,'missing')!='ready'),0),
          COALESCE(SUM(tm.asset_id IS NULL OR tm.inventory_state!='complete'),0),
          COALESCE(SUM(a.protection_state!='replica_verified'),0),
          COALESCE(SUM((SELECT COUNT(*) FROM active_occurrences o WHERE o.asset_id=a.id)>1 AND NOT EXISTS(SELECT 1 FROM duplicate_decisions d WHERE d.asset_id=a.id)),0),
          COALESCE(SUM({FAILURE_PREDICATE}),0)
         FROM assets a
         LEFT JOIN asset_user_state us ON us.asset_id=a.id
         LEFT JOIN thumbnails t ON t.asset_id=a.id
         LEFT JOIN asset_technical_metadata tm ON tm.asset_id=a.id"),
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
    fn failure_pages_and_summary_share_one_asset_based_predicate() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let conn = catalog::open(&root.join("review.sqlite")).unwrap();
        for index in 0..53 {
            let id = format!("a{index:03}");
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?1,?1,'photo','jpg','2026-01-01','exif',1,?1,'2026-01-01')",[&id]).unwrap();
            if index < 51 {
                conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,last_error,updated_at)VALUES(?1,1,'','failed','decoder failed','2026-01-01')",[&id]).unwrap();
            }
            if index == 0 || index == 51 {
                conn.execute("INSERT INTO asset_technical_metadata(asset_id,declared_extension,detected_format,family,support_level,enriched_at,inventory_state,inventory_error)VALUES(?1,'jpg','jpeg','photo','full','2026-01-01','failed','timeout')",[&id]).unwrap();
            }
        }
        let first = failures_from(&conn, -1).unwrap();
        assert_eq!(first.total, 52);
        assert_eq!(first.total, summary_from(&conn).unwrap().technical_failures);
        assert_eq!(first.items.len(), 50);
        assert_eq!(first.next_offset, Some(50));
        assert_eq!(
            first.items[0].preview_error.as_deref(),
            Some("decoder failed")
        );
        assert_eq!(first.items[0].metadata_error.as_deref(), Some("timeout"));
        let second = failures_from(&conn, 50).unwrap();
        assert_eq!(second.items.len(), 2);
        assert_eq!(second.next_offset, None);
        assert!(failures_from(&conn, 500).unwrap().items.is_empty());
        let metadata_only = failures_filtered_from(&conn, 0, "metadata", "a000").unwrap();
        assert_eq!(metadata_only.total, 1);
        assert_eq!(metadata_only.items[0].asset_id, "a000");
        let preview_only = failures_filtered_from(&conn, 0, "preview", "a051").unwrap();
        assert_eq!(preview_only.total, 0);
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
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

    #[test]
    fn undo_restores_an_entire_user_state_batch_atomically() {
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
        for id in ["a", "b"] {
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?1,'photo','jpg','2026-01-01','exif',1,?1,?3)",params![id,id.repeat(64),now]).unwrap();
            conn.execute("INSERT INTO asset_user_state(asset_id,favorite,rating,review_later,description,updated_at)VALUES(?1,1,5,1,'alterado',?2)",params![id,now]).unwrap();
        }
        let undo=serde_json::json!({"items":[{"assetId":"a","favorite":false,"rating":0,"reviewLater":false,"description":""},{"assetId":"b","favorite":false,"rating":2,"reviewLater":false,"description":"antes"}]}).to_string();
        conn.execute("INSERT INTO catalog_actions(id,kind,payload,undo_payload,state,created_at)VALUES('batch','user_state_batch','{}',?1,'applied',?2)",params![undo,now]).unwrap();
        drop(conn);
        assert_eq!(undo_last(&cfg).unwrap().affected, 2);
        let conn = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        let restored:(bool,i64,bool,String)=conn.query_row("SELECT favorite,rating,review_later,description FROM asset_user_state WHERE asset_id='b'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap();
        assert_eq!(restored, (false, 2, false, "antes".into()));
        assert!(conn
            .query_row(
                "SELECT state='undone' FROM catalog_actions WHERE id='batch'",
                [],
                |row| row.get::<_, bool>(0)
            )
            .unwrap());
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
