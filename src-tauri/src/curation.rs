use crate::{gallery, models::GalleryFilters};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurationSession {
    pub id: String,
    pub name: String,
    pub filters: GalleryFilters,
    pub sort: String,
    pub state: String,
    pub total_items: i64,
    pub reviewed_items: i64,
    pub skipped_items: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurationPage {
    pub session: CurationSession,
    pub asset_ids: Vec<String>,
    pub remaining: i64,
}

type RawSession = (
    String,
    String,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    String,
    String,
);

fn row_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawSession> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
    ))
}

fn parse(raw: RawSession) -> Result<CurationSession, String> {
    Ok(CurationSession {
        id: raw.0,
        name: raw.1,
        filters: serde_json::from_str(&raw.2).map_err(|error| error.to_string())?,
        sort: raw.3,
        state: raw.4,
        total_items: raw.5,
        reviewed_items: raw.6,
        skipped_items: raw.7,
        created_at: raw.8,
        updated_at: raw.9,
    })
}

fn find(conn: &Connection, id: &str) -> Result<CurationSession, String> {
    let raw = conn.query_row(
        "SELECT id,name,filters_json,sort,state,total_items,reviewed_items,skipped_items,created_at,updated_at FROM curation_sessions WHERE id=?1",
        [id], row_session,
    ).map_err(|_| "Sessão de curadoria não encontrada".to_string())?;
    parse(raw)
}

pub fn create(
    conn: &mut Connection,
    name: &str,
    filters: GalleryFilters,
    sort: &str,
) -> Result<CurationSession, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Informe um nome de até 100 caracteres".into());
    }
    let ids = gallery::matching_asset_ids(conn, &filters, Some(sort), 50_000)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let json = serde_json::to_string(&filters).map_err(|error| error.to_string())?;
    let state = if ids.is_empty() {
        "completed"
    } else {
        "active"
    };
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO curation_sessions(id,name,filters_json,sort,state,total_items,created_at,updated_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",
        params![id, name, json, sort, state, ids.len() as i64, now],
    ).map_err(|error| error.to_string())?;
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO curation_session_items(session_id,asset_id,position)VALUES(?1,?2,?3)",
            )
            .map_err(|error| error.to_string())?;
        for (position, asset_id) in ids.iter().enumerate() {
            insert
                .execute(params![id, asset_id, position as i64])
                .map_err(|error| error.to_string())?;
        }
    }
    tx.commit().map_err(|error| error.to_string())?;
    find(conn, &id)
}

pub fn list(conn: &Connection) -> Result<Vec<CurationSession>, String> {
    let mut statement = conn.prepare("SELECT id,name,filters_json,sort,state,total_items,reviewed_items,skipped_items,created_at,updated_at FROM curation_sessions ORDER BY state='active' DESC,updated_at DESC").map_err(|error| error.to_string())?;
    let sessions = statement
        .query_map([], row_session)
        .map_err(|error| error.to_string())?
        .map(|row| parse(row.map_err(|error| error.to_string())?))
        .collect();
    sessions
}

pub fn page(conn: &Connection, id: &str) -> Result<CurationPage, String> {
    let session = find(conn, id)?;
    let mut statement = conn.prepare("SELECT asset_id FROM curation_session_items WHERE session_id=?1 AND state='pending' ORDER BY position LIMIT 200").map_err(|error| error.to_string())?;
    let asset_ids = statement
        .query_map([id], |row| row.get(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let remaining = session.total_items - session.reviewed_items - session.skipped_items;
    Ok(CurationPage {
        session,
        asset_ids,
        remaining,
    })
}

pub fn decide(
    conn: &mut Connection,
    id: &str,
    asset_ids: Vec<String>,
    decision: &str,
) -> Result<CurationSession, String> {
    if !matches!(decision, "reviewed" | "skipped" | "pending") {
        return Err("Decisão de curadoria inválida".into());
    }
    let ids = asset_ids.into_iter().take(500).collect::<Vec<_>>();
    if ids.is_empty() {
        return find(conn, id);
    }
    let now = Utc::now().to_rfc3339();
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    for asset_id in ids {
        tx.execute(
            "UPDATE curation_session_items SET state=?3,decided_at=CASE WHEN ?3='pending' THEN NULL ELSE ?4 END WHERE session_id=?1 AND asset_id=?2",
            params![id, asset_id, decision, now],
        ).map_err(|error| error.to_string())?;
    }
    tx.execute(
        "UPDATE curation_sessions SET reviewed_items=(SELECT COUNT(*) FROM curation_session_items WHERE session_id=?1 AND state='reviewed'),skipped_items=(SELECT COUNT(*) FROM curation_session_items WHERE session_id=?1 AND state='skipped'),state=CASE WHEN EXISTS(SELECT 1 FROM curation_session_items WHERE session_id=?1 AND state='pending') THEN 'active' ELSE 'completed' END,updated_at=?2 WHERE id=?1",
        params![id, now],
    ).map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())?;
    find(conn, id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<i64, String> {
    conn.execute("DELETE FROM curation_sessions WHERE id=?1", [id])
        .map(|affected| affected as i64)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use std::fs;

    #[test]
    fn session_is_materialized_and_resumable() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let mut conn = catalog::open(&root.join("curation.sqlite")).unwrap();
        for index in 0..3 {
            let id = format!("a{index}");
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','jpg',?4,'exif',1,?3,?4)", params![id, format!("{index:064}"), format!("{id}.jpg"), format!("2026-01-0{}T00:00:00", index + 1)]).unwrap();
        }
        let session = create(
            &mut conn,
            "Janeiro",
            GalleryFilters::default(),
            "captured_desc",
        )
        .unwrap();
        assert_eq!(session.total_items, 3);
        let first = page(&conn, &session.id).unwrap();
        assert_eq!(first.asset_ids.len(), 3);
        let changed = decide(
            &mut conn,
            &session.id,
            vec![first.asset_ids[0].clone()],
            "reviewed",
        )
        .unwrap();
        assert_eq!(changed.reviewed_items, 1);
        assert_eq!(page(&conn, &session.id).unwrap().remaining, 2);
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
