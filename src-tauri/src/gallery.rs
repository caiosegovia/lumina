use crate::models::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rusqlite::{params_from_iter, types::Value, Connection};
#[cfg(test)]
static RELATION_QUERIES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Cursor {
    captured_at: String,
    id: String,
}
fn add(c: &mut Vec<String>, v: &mut Vec<Value>, sql: &str, value: Value) {
    v.push(value);
    c.push(sql.replace('?', &format!("?{}", v.len())))
}
fn conditions(f: &GalleryFilters) -> (Vec<String>, Vec<Value>) {
    let (mut c, mut v) = (Vec::new(), Vec::new());
    if !f.query.trim().is_empty() {
        let x = Value::Text(format!("%{}%", f.query.trim().to_lowercase()));
        v.extend([x.clone(), x.clone(), x]);
        let n = v.len();
        c.push(format!("(EXISTS(SELECT 1 FROM assets_fts sf WHERE sf.asset_id=a.id AND (sf.filename LIKE ?{} OR sf.camera LIKE ?{})) OR EXISTS(SELECT 1 FROM asset_tags aq JOIN tags tq ON tq.id=aq.tag_id WHERE aq.asset_id=a.id AND LOWER(tq.name) LIKE ?{n}))",n-2,n-1));
    }
    if let Some(x) = f.year {
        add(
            &mut c,
            &mut v,
            "a.captured_at>=?",
            Value::Text(format!("{x:04}-01-01T00:00:00")),
        );
        add(
            &mut c,
            &mut v,
            "a.captured_at<?",
            Value::Text(format!("{:04}-01-01T00:00:00", x + 1)),
        );
    }
    if let Some(x) = f.date_from.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "a.captured_at>=?",
            Value::Text(format!("{x}T00:00:00")),
        )
    }
    if let Some(x) = f.date_to.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "a.captured_at<?",
            Value::Text(
                chrono::NaiveDate::parse_from_str(x, "%Y-%m-%d")
                    .ok()
                    .and_then(|date| date.succ_opt())
                    .map(|date| format!("{date}T00:00:00"))
                    .unwrap_or_else(|| format!("{x}T23:59:59.999")),
            ),
        )
    }
    if let Some(x) = f.media_type.as_ref().filter(|x| !x.is_empty()) {
        add(&mut c, &mut v, "a.media_type=?", Value::Text(x.clone()))
    }
    if let Some(x) = f.camera.as_ref().filter(|x| !x.is_empty()) {
        add(&mut c, &mut v, "a.camera=?", Value::Text(x.clone()))
    }
    if let Some(x) = f.source_id.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "EXISTS(SELECT 1 FROM occurrences os WHERE os.asset_id=a.id AND os.source_id=?)",
            Value::Text(x.clone()),
        )
    }
    if let Some(x) = f.original_folder.as_ref().filter(|x| !x.trim().is_empty()) {
        add(
            &mut c,
            &mut v,
            "EXISTS(SELECT 1 FROM occurrences op WHERE op.asset_id=a.id AND LOWER(op.path) LIKE ?)",
            Value::Text(format!("%{}%", x.trim().to_lowercase())),
        )
    }
    if let Some(x) = f.extension.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "a.extension=?",
            Value::Text(x.to_ascii_lowercase()),
        )
    }
    if let Some(x) = f.has_location {
        c.push(
            if x {
                "a.latitude IS NOT NULL AND a.longitude IS NOT NULL"
            } else {
                "(a.latitude IS NULL OR a.longitude IS NULL)"
            }
            .into(),
        )
    }
    if let Some(x) = f.tag_id.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "EXISTS(SELECT 1 FROM asset_tags atf WHERE atf.asset_id=a.id AND atf.tag_id=?)",
            Value::Text(x.clone()),
        )
    }
    if let Some(x) = f.album_id.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "EXISTS(SELECT 1 FROM album_assets aaf WHERE aaf.asset_id=a.id AND aaf.album_id=?)",
            Value::Text(x.clone()),
        )
    }
    if let Some(x) = f.protection_state.as_ref().filter(|x| !x.is_empty()) {
        add(
            &mut c,
            &mut v,
            "a.protection_state=?",
            Value::Text(x.clone()),
        )
    }
    if f.date_suspicious == Some(true) {
        c.push("(a.date_source IN ('file','fallback','filesystem') OR CAST(substr(a.captured_at,1,4) AS INTEGER)<1990 OR CAST(substr(a.captured_at,1,4) AS INTEGER)>CAST(strftime('%Y','now') AS INTEGER)+1)".into())
    }
    if let Some(x) = f.favorite {
        c.push(format!(
            "COALESCE((SELECT favorite FROM asset_user_state us WHERE us.asset_id=a.id),0)={}",
            if x { 1 } else { 0 }
        ));
    }
    if let Some(x) = f.minimum_rating.filter(|value| *value > 0) {
        add(
            &mut c,
            &mut v,
            "COALESCE((SELECT rating FROM asset_user_state us WHERE us.asset_id=a.id),0)>=?",
            Value::Integer(x.clamp(1, 5)),
        );
    }
    if let Some(x) = f.review_later {
        c.push(format!(
            "COALESCE((SELECT review_later FROM asset_user_state us WHERE us.asset_id=a.id),0)={}",
            if x { 1 } else { 0 }
        ));
    }
    (c, v)
}
fn where_sql(c: &[String]) -> String {
    if c.is_empty() {
        "1=1".into()
    } else {
        c.join(" AND ")
    }
}
fn opts(conn: &Connection, sql: &str) -> Result<Vec<FilterOption>, String> {
    let mut s = conn.prepare(sql).map_err(|e| e.to_string())?;
    let r = s
        .query_map([], |x| {
            Ok(FilterOption {
                value: x.get(0)?,
                label: x.get(1)?,
                count: x.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(r)
}
fn options(c: &Connection) -> Result<GalleryFilterOptions, String> {
    Ok(GalleryFilterOptions{cameras:opts(c,"SELECT camera,camera,COUNT(*) FROM assets WHERE camera IS NOT NULL AND camera!='' GROUP BY camera ORDER BY COUNT(*) DESC,camera")?,sources:opts(c,"SELECT s.id,s.name,COUNT(DISTINCT o.asset_id) FROM sources s JOIN active_occurrences o ON o.source_id=s.id GROUP BY s.id ORDER BY COUNT(DISTINCT o.asset_id) DESC,s.name")?,extensions:opts(c,"SELECT LOWER(extension),UPPER(extension),COUNT(*) FROM assets GROUP BY LOWER(extension) ORDER BY COUNT(*) DESC,extension")?,tags:opts(c,"SELECT t.id,t.name,COUNT(at.asset_id) FROM tags t JOIN asset_tags at ON at.tag_id=t.id GROUP BY t.id ORDER BY COUNT(at.asset_id) DESC,t.name")?,albums:opts(c,"SELECT al.id,al.name,COUNT(aa.asset_id) FROM albums al JOIN album_assets aa ON aa.album_id=al.id GROUP BY al.id ORDER BY COUNT(aa.asset_id) DESC,al.name")?})
}

fn page_relations(
    conn: &Connection,
    asset_ids: &[String],
    relation: &str,
) -> Result<HashMap<String, Vec<String>>, String> {
    if asset_ids.is_empty() {
        return Ok(HashMap::new());
    }
    #[cfg(test)]
    RELATION_QUERIES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let placeholders = (1..=asset_ids.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = match relation {
        "sources" => format!("SELECT o.asset_id,s.name FROM active_occurrences o JOIN sources s ON s.id=o.source_id WHERE o.asset_id IN({placeholders}) GROUP BY o.asset_id,s.name ORDER BY o.asset_id,s.name"),
        "tags" => format!("SELECT at.asset_id,t.name FROM asset_tags at JOIN tags t ON t.id=at.tag_id WHERE at.asset_id IN({placeholders}) ORDER BY at.asset_id,t.name"),
        _ => return Err("Relação de galeria inválida".into()),
    };
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let pairs = statement
        .query_map(params_from_iter(asset_ids.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut grouped = HashMap::<String, Vec<String>>::new();
    for (asset, value) in pairs {
        grouped.entry(asset).or_default().push(value);
    }
    Ok(grouped)
}

pub fn search(conn: &Connection, r: &GalleryRequest) -> Result<GalleryResult, String> {
    let (mut clauses, mut values) = conditions(&r.filters);
    let base = where_sql(&clauses);
    let (t, years, filter_options) = if r.cursor.is_none() {
        let q=format!("SELECT COUNT(*),COALESCE(SUM(a.bytes),0),COALESCE(SUM(a.media_type='photo'),0),COALESCE(SUM(a.media_type='video'),0),COALESCE(SUM(a.media_type='raw'),0),COALESCE(SUM(a.protection_state='replica_verified'),0),COALESCE(SUM(a.latitude IS NOT NULL AND a.longitude IS NOT NULL),0),COALESCE(SUM((SELECT COUNT(*) FROM active_occurrences od WHERE od.asset_id=a.id)>1),0),COALESCE(SUM(COALESCE((SELECT favorite FROM asset_user_state us WHERE us.asset_id=a.id),0)),0),COALESCE(SUM(COALESCE((SELECT inventory_state!='complete' FROM asset_technical_metadata tm WHERE tm.asset_id=a.id),1)),0),COALESCE(SUM(a.protection_state!='replica_verified'),0) FROM assets a WHERE {base}");
        let totals = conn
            .query_row(&q, params_from_iter(values.iter()), |x| {
                Ok((
                    x.get(0)?,
                    x.get(1)?,
                    x.get(2)?,
                    x.get(3)?,
                    x.get(4)?,
                    x.get(5)?,
                    x.get(6)?,
                    x.get(7)?,
                    x.get(8)?,
                    x.get(9)?,
                    x.get(10)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let q=format!("SELECT CASE WHEN substr(a.captured_at,1,4) GLOB '[0-9][0-9][0-9][0-9]' THEN substr(a.captured_at,1,4) ELSE 'Sem data' END,COUNT(*),COALESCE(SUM(a.bytes),0) FROM assets a WHERE {base} GROUP BY 1 ORDER BY 1 DESC");
        let mut statement = conn.prepare(&q).map_err(|e| e.to_string())?;
        let years = statement
            .query_map(params_from_iter(values.iter()), |x| {
                Ok(GalleryYearCount {
                    year: x.get(0)?,
                    count: x.get(1)?,
                    bytes: x.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        (totals, years, options(conn)?)
    } else {
        (
            (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0),
            Vec::new(),
            GalleryFilterOptions::default(),
        )
    };
    if let Some(x) = &r.cursor {
        let raw = URL_SAFE_NO_PAD
            .decode(x)
            .map_err(|_| "Cursor inválido".to_string())?;
        let cur: Cursor =
            serde_json::from_slice(&raw).map_err(|_| "Cursor inválido".to_string())?;
        values.push(Value::Text(cur.captured_at));
        let n = values.len();
        values.push(Value::Text(cur.id));
        clauses.push(format!(
            "(a.captured_at<?{n} OR (a.captured_at=?{n} AND a.id<?{}))",
            n + 1
        ));
    }
    let limit = r.limit.unwrap_or(100).clamp(1, 200) as usize;
    let q=format!("SELECT a.id,a.filename,a.media_type,a.extension,a.captured_at,a.date_source,a.bytes,a.width,a.height,a.duration,a.camera,a.latitude,a.longitude,a.master_path,a.hash,a.protection_state,(SELECT COUNT(*) FROM active_occurrences o WHERE o.asset_id=a.id),COALESCE((SELECT favorite FROM asset_user_state us WHERE us.asset_id=a.id),0),COALESCE((SELECT rating FROM asset_user_state us WHERE us.asset_id=a.id),0),COALESCE((SELECT review_later FROM asset_user_state us WHERE us.asset_id=a.id),0),COALESCE((SELECT description FROM asset_user_state us WHERE us.asset_id=a.id),'') FROM assets a WHERE {} ORDER BY a.captured_at DESC,a.id DESC LIMIT {}",where_sql(&clauses),limit+1);
    let mut s = conn.prepare(&q).map_err(|e| e.to_string())?;
    let mut rows = s
        .query_map(params_from_iter(values.iter()), |x| {
            Ok((
                x.get::<_, String>(0)?,
                x.get::<_, String>(1)?,
                x.get::<_, String>(2)?,
                x.get::<_, String>(3)?,
                x.get::<_, String>(4)?,
                x.get::<_, String>(5)?,
                x.get::<_, i64>(6)?,
                x.get::<_, Option<i64>>(7)?,
                x.get::<_, Option<i64>>(8)?,
                x.get::<_, Option<f64>>(9)?,
                x.get::<_, Option<String>>(10)?,
                x.get::<_, Option<f64>>(11)?,
                x.get::<_, Option<f64>>(12)?,
                x.get::<_, String>(13)?,
                x.get::<_, String>(14)?,
                x.get::<_, String>(15)?,
                x.get::<_, i64>(16)?,
                x.get::<_, bool>(17)?,
                x.get::<_, i64>(18)?,
                x.get::<_, bool>(19)?,
                x.get::<_, String>(20)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(s);
    let more = rows.len() > limit;
    if more {
        rows.pop();
    }
    let next_cursor = if more {
        rows.last().map(|x| {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&Cursor {
                    captured_at: x.4.clone(),
                    id: x.0.clone(),
                })
                .unwrap(),
            )
        })
    } else {
        None
    };
    let asset_ids = rows.iter().map(|row| row.0.clone()).collect::<Vec<_>>();
    let mut sources = page_relations(conn, &asset_ids, "sources")?;
    let mut tags = page_relations(conn, &asset_ids, "tags")?;
    let assets = rows
        .into_iter()
        .map(|x| {
            let year =
                x.4.get(0..4)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            let date_suspicious = matches!(x.5.as_str(), "file" | "fallback" | "filesystem")
                || year < 1990
                || year
                    > chrono::Utc::now()
                        .format("%Y")
                        .to_string()
                        .parse::<i32>()
                        .unwrap_or(9999)
                        + 1;
            MediaAsset {
                id: x.0.clone(),
                filename: x.1,
                media_type: x.2,
                extension: x.3,
                captured_at: x.4,
                date_source: x.5,
                date_suspicious,
                bytes: x.6,
                width: x.7,
                height: x.8,
                duration: x.9,
                camera: x.10,
                latitude: x.11,
                longitude: x.12,
                thumbnail: None,
                master_path: x.13,
                hash: x.14,
                protection_state: x.15,
                occurrence_count: x.16,
                source_names: sources.remove(&x.0).unwrap_or_default(),
                tags: tags.remove(&x.0).unwrap_or_default(),
                favorite: x.17,
                rating: x.18,
                review_later: x.19,
                description: x.20,
            }
        })
        .collect();
    Ok(GalleryResult {
        assets,
        matched: t.0,
        next_cursor,
        summary: GallerySummary {
            total: t.0,
            bytes: t.1,
            photos: t.2,
            videos: t.3,
            raw: t.4,
            protected: t.5,
            with_location: t.6,
            duplicate_assets: t.7,
            favorites: t.8,
            incomplete_metadata: t.9,
            pending_protection: t.10,
            years,
        },
        options: filter_options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use rusqlite::params;
    use std::fs;
    use uuid::Uuid;
    fn seed() -> (std::path::PathBuf, Connection) {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        fs::create_dir_all(&root).unwrap();
        let db = catalog::open(&root.join("g.sqlite")).unwrap();
        db.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('s','Drone','E:/DCIM','C')",
            [],
        )
        .unwrap();
        for (id, k, d, cam, lat) in [
            ("a", "photo", "2024-02-01T00:00:00Z", "Canon", Some(1.)),
            ("b", "video", "2025-01-01T00:00:00Z", "DJI", None),
            ("c", "raw", "2024-01-01T00:00:00Z", "Canon", None),
        ] {
            db.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,camera,latitude,longitude,master_path,created_at)VALUES(?1,?2,?3,?4,'jpg',?5,'exif',100,?6,?7,?7,'master',?5)",params![id,format!("{id:0<64}"),format!("{id}.jpg"),k,d,cam,lat]).unwrap();
            db.execute("INSERT INTO occurrences(id,asset_id,source_id,path,seen_at)VALUES(?1,?2,'s',?3,?4)",params![format!("o{id}"),id,format!("E:/{id}"),d]).unwrap();
        }
        (root, db)
    }
    #[test]
    fn filters_and_stats() {
        let (r, d) = seed();
        let x = search(
            &d,
            &GalleryRequest {
                filters: GalleryFilters {
                    year: Some(2024),
                    media_type: Some("photo".into()),
                    has_location: Some(true),
                    ..Default::default()
                },
                cursor: None,
                limit: Some(20),
            },
        )
        .unwrap();
        assert_eq!(x.matched, 1);
        assert_eq!(x.assets[0].id, "a");
        drop(d);
        fs::remove_dir_all(r).unwrap()
    }
    #[test]
    fn personal_organization_is_persisted_and_filterable() {
        let (root, db) = seed();
        db.execute(
            "INSERT INTO asset_user_state(asset_id,favorite,rating,review_later,description,updated_at)VALUES('a',1,5,1,'Viagem especial','2026-01-01')",
            [],
        )
        .unwrap();
        let result = search(
            &db,
            &GalleryRequest {
                filters: GalleryFilters {
                    favorite: Some(true),
                    minimum_rating: Some(4),
                    review_later: Some(true),
                    ..Default::default()
                },
                cursor: None,
                limit: Some(20),
            },
        )
        .unwrap();
        assert_eq!(result.matched, 1);
        assert_eq!(result.assets[0].id, "a");
        assert_eq!(result.assets[0].rating, 5);
        assert_eq!(result.assets[0].description, "Viagem especial");
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn page_relations_use_two_batched_queries_instead_of_n_plus_one() {
        let (root, db) = seed();
        RELATION_QUERIES.store(0, std::sync::atomic::Ordering::SeqCst);
        let result = search(
            &db,
            &GalleryRequest {
                filters: Default::default(),
                cursor: None,
                limit: Some(100),
            },
        )
        .unwrap();
        assert_eq!(result.assets.len(), 3);
        assert_eq!(
            RELATION_QUERIES.load(std::sync::atomic::Ordering::SeqCst),
            2
        );
        assert!(result
            .assets
            .iter()
            .all(|asset| asset.source_names == vec!["Drone"]));
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cursor_no_overlap() {
        let (r, d) = seed();
        let a = search(
            &d,
            &GalleryRequest {
                filters: Default::default(),
                cursor: None,
                limit: Some(2),
            },
        )
        .unwrap();
        let b = search(
            &d,
            &GalleryRequest {
                filters: Default::default(),
                cursor: a.next_cursor,
                limit: Some(2),
            },
        )
        .unwrap();
        assert_eq!(b.assets.len(), 1);
        assert!(!a.assets.iter().any(|x| x.id == b.assets[0].id));
        drop(d);
        fs::remove_dir_all(r).unwrap()
    }
    #[test]
    fn timeline_filter_uses_composite_index() {
        let (root, db) = seed();
        let detail:String=db.query_row("EXPLAIN QUERY PLAN SELECT id FROM assets WHERE media_type='photo' AND captured_at>='2024-01-01T00:00:00' AND captured_at<'2025-01-01T00:00:00' ORDER BY captured_at DESC,id DESC LIMIT 100",[],|row|row.get(3)).unwrap();
        assert!(
            detail.contains("idx_assets_type_timeline"),
            "plano inesperado: {detail}"
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn input_is_bound() {
        let (c, v) = conditions(&GalleryFilters {
            query: "%' OR 1=1 --".into(),
            ..Default::default()
        });
        assert!(!where_sql(&c).contains("1=1 --"));
        assert_eq!(v.len(), 3)
    }
    #[test]
    fn suspicious_date_filter_is_explicit_and_parameter_free() {
        let (clauses, values) = conditions(&GalleryFilters {
            date_suspicious: Some(true),
            ..Default::default()
        });
        assert!(clauses.iter().any(|x| x.contains("date_source")));
        assert!(values.is_empty());
    }
}
