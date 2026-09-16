use crate::{catalog, models::LibraryConfig};
use chrono::Utc;
use rusqlite::{params, params_from_iter, types::Value};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};

const ALGORITHM_VERSION: i64 = 1;
const SAMPLE_PER_STRATUM: i64 = 12;
static CANCEL_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightRequest {
    pub year: Option<i32>,
    pub month: Option<u8>,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightCard {
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub value: i64,
    pub action: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightReport {
    pub scope_key: String,
    pub mode: String,
    pub sampled_items: i64,
    pub total_items: i64,
    pub coverage_percent: f64,
    pub generated_at: String,
    pub duration_ms: i64,
    pub cached: bool,
    pub cards: Vec<InsightCard>,
}

#[derive(Debug)]
struct Item {
    media: String,
    captured: String,
    camera: Option<String>,
    protected: bool,
    review: bool,
    located: bool,
}

fn validate(request: &InsightRequest) -> Result<String, String> {
    if request.mode != "sample" && request.mode != "full" {
        return Err("Modo de análise inválido".into());
    }
    if request.month.is_some() && request.year.is_none() {
        return Err("O mês exige um ano".into());
    }
    if let Some(month) = request.month {
        if !(1..=12).contains(&month) {
            return Err("Mês inválido".into());
        }
    }
    Ok(match (request.year, request.month) {
        (Some(y), Some(m)) => format!("{y:04}-{m:02}"),
        (Some(y), None) => format!("{y:04}"),
        _ => "all".into(),
    })
}

pub fn cancel() {
    CANCEL_REQUESTED.store(true, Ordering::Release);
}

pub fn generate(cfg: &LibraryConfig, request: InsightRequest) -> Result<InsightReport, String> {
    let scope = validate(&request)?;
    CANCEL_REQUESTED.store(false, Ordering::Release);
    let started = Instant::now();
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let prefix = if scope == "all" {
        "%".to_string()
    } else {
        format!("{scope}%")
    };
    let fingerprint: String = conn.query_row("SELECT printf('%d:%s',COUNT(*),COALESCE(MAX(created_at),'')) FROM assets WHERE captured_at LIKE ?1", [&prefix], |r| r.get(0)).map_err(|e|e.to_string())?;
    let cache_key = format!("v{ALGORITHM_VERSION}:{scope}:{}", request.mode);
    if let Some(payload) = conn.query_row("SELECT payload FROM insight_runs WHERE cache_key=?1 AND catalog_fingerprint=?2 AND state='completed'", params![cache_key,fingerprint], |r|r.get::<_,String>(0)).optional().map_err(|e|e.to_string())? {
        let mut report: InsightReport=serde_json::from_str(&payload).map_err(|e|e.to_string())?; report.cached=true; return Ok(report);
    }
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM assets WHERE captured_at LIKE ?1",
            [&prefix],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let mut values = vec![Value::from(prefix)];
    let limit = if request.mode == "sample" {
        "WHERE rn<=?2"
    } else {
        ""
    };
    if request.mode == "sample" {
        values.push(Value::from(SAMPLE_PER_STRATUM));
    }
    let sql=format!("WITH ranked AS(SELECT a.media_type,a.captured_at,a.camera,a.protection_state,COALESCE(u.review_later,0) AS review_later,CASE WHEN a.latitude IS NOT NULL AND a.longitude IS NOT NULL THEN 1 ELSE 0 END AS located,ROW_NUMBER()OVER(PARTITION BY substr(a.captured_at,1,7),a.media_type,COALESCE(a.camera,'') ORDER BY a.id)rn FROM assets a LEFT JOIN asset_user_state u ON u.asset_id=a.id WHERE a.captured_at LIKE ?1)SELECT media_type,captured_at,camera,protection_state,review_later,located FROM ranked {limit}");
    let mut statement = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params_from_iter(values), |r| {
            Ok(Item {
                media: r.get(0)?,
                captured: r.get(1)?,
                camera: r.get(2)?,
                protected: r.get::<_, String>(3)? == "replica_verified",
                review: r.get::<_, i64>(4)? != 0,
                located: r.get::<_, i64>(5)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for row in rows {
        if CANCEL_REQUESTED.load(Ordering::Acquire) {
            return Err("INSIGHT_CANCELED".into());
        }
        items.push(row.map_err(|e| e.to_string())?);
    }
    let sampled = items.len() as i64;
    let photos = items.iter().filter(|x| x.media != "video").count() as i64;
    let videos = sampled - photos;
    let pending = items.iter().filter(|x| !x.protected).count() as i64;
    let review = items.iter().filter(|x| x.review).count() as i64;
    let located = items.iter().filter(|x| x.located).count() as i64;
    let mut cameras = HashMap::new();
    for item in &items {
        if let Some(c) = item.camera.as_ref().filter(|x| !x.trim().is_empty()) {
            *cameras.entry(c.clone()).or_insert(0i64) += 1;
        }
    }
    let top_camera = cameras.into_iter().max_by_key(|x| x.1);
    let first = items
        .iter()
        .map(|x| x.captured.as_str())
        .min()
        .unwrap_or("");
    let last = items
        .iter()
        .map(|x| x.captured.as_str())
        .max()
        .unwrap_or("");
    let confidence = if request.mode == "full" {
        "alta"
    } else if total > 0 && sampled * 100 / total >= 30 {
        "média"
    } else {
        "indicativa"
    };
    let mut cards = vec![
        InsightCard {
            kind: "composition".into(),
            title: "Composição do período".into(),
            detail: format!("{photos} fotos e {videos} vídeos"),
            value: sampled,
            action: "library".into(),
            confidence: confidence.into(),
        },
        InsightCard {
            kind: "timeline".into(),
            title: "Intervalo registrado".into(),
            detail: if first.is_empty() {
                "Sem datas no recorte".into()
            } else {
                format!(
                    "{} até {}",
                    &first[..10.min(first.len())],
                    &last[..10.min(last.len())]
                )
            },
            value: sampled,
            action: "library".into(),
            confidence: confidence.into(),
        },
        InsightCard {
            kind: "protection".into(),
            title: "Proteção pendente".into(),
            detail: format!("{pending} itens da análise ainda não têm réplica verificada"),
            value: pending,
            action: "protection".into(),
            confidence: confidence.into(),
        },
        InsightCard {
            kind: "review".into(),
            title: "Curadoria pendente".into(),
            detail: format!("{review} itens marcados para revisar"),
            value: review,
            action: "review".into(),
            confidence: confidence.into(),
        },
        InsightCard {
            kind: "location".into(),
            title: "Cobertura geográfica".into(),
            detail: format!("{located} de {sampled} itens analisados possuem GPS"),
            value: located,
            action: "discover".into(),
            confidence: confidence.into(),
        },
    ];
    if let Some((camera, count)) = top_camera {
        cards.push(InsightCard {
            kind: "equipment".into(),
            title: "Equipamento predominante".into(),
            detail: camera,
            value: count,
            action: "library".into(),
            confidence: confidence.into(),
        });
    }
    let report = InsightReport {
        scope_key: scope,
        mode: request.mode,
        sampled_items: sampled,
        total_items: total,
        coverage_percent: if total == 0 {
            100.0
        } else {
            sampled as f64 * 100.0 / total as f64
        },
        generated_at: Utc::now().to_rfc3339(),
        duration_ms: started.elapsed().as_millis() as i64,
        cached: false,
        cards,
    };
    let payload = serde_json::to_string(&report).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO insight_runs(cache_key,scope_key,mode,algorithm_version,catalog_fingerprint,state,sampled_items,total_items,payload,generated_at)VALUES(?1,?2,?3,?4,?5,'completed',?6,?7,?8,?9)ON CONFLICT(cache_key)DO UPDATE SET catalog_fingerprint=excluded.catalog_fingerprint,state='completed',sampled_items=excluded.sampled_items,total_items=excluded.total_items,payload=excluded.payload,generated_at=excluded.generated_at",params![cache_key,report.scope_key,report.mode,ALGORITHM_VERSION,fingerprint,report.sampled_items,report.total_items,payload,report.generated_at]).map_err(|e|e.to_string())?;
    Ok(report)
}

use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    fn fixture() -> (std::path::PathBuf, LibraryConfig) {
        let root = std::env::temp_dir().join(format!("lumina-insights-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join(".lumina")).unwrap();
        let cfg = LibraryConfig {
            id: "test".into(),
            name: "test".into(),
            master_path: root.to_string_lossy().into(),
            backup_path: root.join("backup").to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let db = catalog::open(&root.join(".lumina/catalog.sqlite")).unwrap();
        for (id, date, media, camera, protected) in [
            (
                "a",
                "2025-01-02T10:00:00",
                "photo",
                "Phone",
                "replica_verified",
            ),
            ("b", "2025-02-03T10:00:00", "video", "Drone", "consolidated"),
            ("c", "2026-01-01T10:00:00", "photo", "Phone", "consolidated"),
        ] {
            db.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,camera,master_path,protection_state,created_at)VALUES(?1,?2,?3,?4,'jpg',?5,'test',100,?6,?7,?8,datetime('now'))",params![id,format!("hash-{id}"),format!("{id}.jpg"),media,date,camera,root.join(format!("{id}.jpg")).to_string_lossy(),protected]).unwrap();
        }
        (root, cfg)
    }

    #[test]
    fn reports_are_scoped_cached_and_do_not_open_originals() {
        let (root, cfg) = fixture();
        let first = generate(
            &cfg,
            InsightRequest {
                year: Some(2025),
                month: None,
                mode: "sample".into(),
            },
        )
        .unwrap();
        assert_eq!(first.total_items, 2);
        assert_eq!(first.sampled_items, 2);
        assert!(!first.cached);
        let cached = generate(
            &cfg,
            InsightRequest {
                year: Some(2025),
                month: None,
                mode: "sample".into(),
            },
        )
        .unwrap();
        assert!(cached.cached);
        assert!(cached
            .cards
            .iter()
            .any(|card| card.kind == "protection" && card.value == 1));
        let full = generate(
            &cfg,
            InsightRequest {
                year: None,
                month: None,
                mode: "full".into(),
            },
        )
        .unwrap();
        assert_eq!(full.total_items, 3);
        assert_eq!(full.coverage_percent, 100.0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_scope() {
        let (_root, cfg) = fixture();
        assert!(generate(
            &cfg,
            InsightRequest {
                year: None,
                month: Some(2),
                mode: "sample".into()
            }
        )
        .is_err());
    }
}
