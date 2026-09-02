mod backup;
mod catalog;
mod diagnostics;
mod duplicates;
mod engine;
mod events;
mod formats;
mod gallery;
mod health;
mod jobs;
mod library;
mod media;
mod metadata;
mod models;
mod pipeline;
mod process;
mod resource;
mod review;
mod storage;
mod sync;
mod volume;

use chrono::Utc;
use models::*;
use rusqlite::{params, OptionalExtension};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState {
    library: Mutex<Option<LibraryConfig>>,
    config_path: PathBuf,
    library_lock: Mutex<Option<library::LibraryLock>>,
}
fn db(cfg: &LibraryConfig) -> Result<rusqlite::Connection, String> {
    catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())
}
fn current(state: &State<AppState>) -> Result<LibraryConfig, String> {
    if state
        .library_lock
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?
        .is_none()
    {
        return Err("Esta biblioteca já está aberta por outra instância do Lumina.".into());
    }
    state
        .library
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())?
        .clone()
        .ok_or_else(|| "Nenhuma biblioteca configurada".to_string())
}

#[tauri::command]
fn get_library(state: State<AppState>) -> Option<LibraryConfig> {
    state.library.lock().ok().and_then(|v| v.clone())
}
fn persist_config(state: &State<AppState>, cfg: &LibraryConfig) -> Result<(), String> {
    fs::write(
        &state.config_path,
        serde_json::to_vec_pretty(cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
#[tauri::command]
fn update_backup_path(
    backup_path: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<LibraryConfig, String> {
    if manager.has_active() {
        return Err("Aguarde o trabalho atual terminar antes de alterar o destino".into());
    }
    fs::create_dir_all(&backup_path).map_err(|e| e.to_string())?;
    let mut cfg = current(&state)?;
    let master = fs::canonicalize(&cfg.master_path).map_err(|e| e.to_string())?;
    let backup = fs::canonicalize(&backup_path).map_err(|e| e.to_string())?;
    if master == backup || master.starts_with(&backup) || backup.starts_with(&master) {
        return Err("A réplica precisa ficar fora da pasta do acervo".into());
    }
    cfg.backup_path = backup.to_string_lossy().into();
    let conn = db(&cfg)?;
    let maintenance = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT OR IGNORE INTO sources(id,name,path,volume_label,available)VALUES('_lumina_maintenance','Manutenção da biblioteca','lumina://maintenance','internal',1)",[]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at,library_state,backup_state)VALUES(?1,'_lumina_maintenance','lumina://maintenance','protection_pending','protection_pending',?2,?2,'verified','pending')",params![maintenance,now]).map_err(|e|e.to_string())?;
    conn.execute(
        "UPDATE assets SET protection_state='stale' WHERE protection_state='replica_verified'",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("UPDATE backup_entries SET state='stale'", [])
        .map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO work_queue(job_id,asset_id,kind,state,created_at,updated_at)SELECT ?1,id,'backup','pending',?2,?2 FROM assets",params![maintenance,now]).map_err(|e|e.to_string())?;
    persist_config(&state, &cfg)?;
    *state
        .library
        .lock()
        .map_err(|_| "Estado interno indisponível")? = Some(cfg.clone());
    Ok(cfg)
}
#[tauri::command]
async fn migrate_master_path(
    new_master_path: String,
    state: State<'_, AppState>,
    manager: State<'_, jobs::JobManager>,
) -> Result<MigrationProgress, String> {
    if manager.has_active() {
        return Err("Aguarde o trabalho atual terminar antes de migrar o acervo".into());
    }
    let cfg = current(&state)?;
    let new_path = PathBuf::from(new_master_path);
    let cfg_for_work = cfg.clone();
    let path_for_work = new_path.clone();
    let progress = tauri::async_runtime::spawn_blocking(move || {
        library::migrate_master(&cfg_for_work, &path_for_work)
    })
    .await
    .map_err(|e| e.to_string())??;
    let mut next = cfg;
    next.master_path = progress.new_master.clone();
    let guard =
        library::LibraryLock::acquire(Path::new(&next.master_path), &Uuid::new_v4().to_string())?;
    persist_config(&state, &next)?;
    *state
        .library
        .lock()
        .map_err(|_| "Estado interno indisponível")? = Some(next);
    *state
        .library_lock
        .lock()
        .map_err(|_| "Estado interno indisponível")? = Some(guard);
    Ok(progress)
}
#[tauri::command]
fn frontend_ready(window: tauri::Window) -> Result<(), String> {
    window.set_title("Lumina Ready").map_err(|e| e.to_string())
}
#[tauri::command]
fn create_library(
    name: String,
    master_path: String,
    backup_path: String,
    state: State<AppState>,
) -> Result<LibraryConfig, String> {
    if master_path.trim().is_empty() || backup_path.trim().is_empty() {
        return Err("Informe as pastas do acervo e do backup".into());
    }
    if Path::new(&master_path) == Path::new(&backup_path) {
        return Err("O acervo e o backup precisam usar pastas diferentes".into());
    }
    fs::create_dir_all(Path::new(&master_path).join(".lumina"))
        .map_err(|e| format!("Não foi possível criar o acervo: {e}"))?;
    fs::create_dir_all(&backup_path)
        .map_err(|e| format!("Não foi possível criar o backup: {e}"))?;
    let master_abs = fs::canonicalize(&master_path).map_err(|e| e.to_string())?;
    let backup_abs = fs::canonicalize(&backup_path).map_err(|e| e.to_string())?;
    if master_abs.starts_with(&backup_abs) || backup_abs.starts_with(&master_abs) {
        return Err("As pastas do acervo e do backup não podem estar contidas uma na outra".into());
    }
    let cfg = LibraryConfig {
        id: Uuid::new_v4().to_string(),
        name,
        master_path,
        backup_path,
        created_at: Utc::now().to_rfc3339(),
    };
    let guard =
        library::LibraryLock::acquire(Path::new(&cfg.master_path), &Uuid::new_v4().to_string())?;
    db(&cfg)?;
    if let Some(parent) = state.config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?
    }
    fs::write(
        &state.config_path,
        serde_json::to_vec_pretty(&cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    *state
        .library
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())? = Some(cfg.clone());
    *state
        .library_lock
        .lock()
        .map_err(|_| "Estado interno indisponível".to_string())? = Some(guard);
    Ok(cfg)
}
#[tauri::command]
async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || get_dashboard_sync(&cfg))
        .await
        .map_err(|error| error.to_string())?
}

fn get_dashboard_sync(cfg: &LibraryConfig) -> Result<DashboardStats, String> {
    let conn = db(cfg)?;
    let cached = conn.query_row("SELECT payload,invalidated_at IS NOT NULL FROM dashboard_snapshots WHERE id=1 AND schema_version=2",[],|row|Ok((row.get::<_,String>(0)?,row.get::<_,bool>(1)?))).optional().map_err(|error|error.to_string())?;
    if let Some((payload, stale)) = cached {
        let mut result: DashboardStats =
            serde_json::from_str(&payload).map_err(|error| error.to_string())?;
        result.stale = stale;
        return Ok(result);
    }
    drop(conn);
    let result = quick_dashboard(cfg)?;
    Ok(result)
}

fn quick_dashboard(cfg: &LibraryConfig) -> Result<DashboardStats, String> {
    let started = std::time::Instant::now();
    let conn = db(cfg)?;
    let read = |dimension: &str| -> Result<Vec<DashboardBreakdown>, String> {
        let mut statement = conn.prepare("SELECT key,items,bytes FROM library_rollups WHERE dimension=?1 AND items>0 ORDER BY bytes DESC").map_err(|error| error.to_string())?;
        let values = statement
            .query_map([dimension], |row| {
                Ok(DashboardBreakdown {
                    key: row.get(0)?,
                    items: row.get(1)?,
                    bytes: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(values)
    };
    let types = read("type")?;
    let years = read("year")?;
    let protection = read("protection")?;
    let extensions = read("extension")?;
    let total_assets = types.iter().map(|value| value.items).sum();
    let bytes = types.iter().map(|value| value.bytes).sum();
    let videos = types
        .iter()
        .find(|value| value.key == "video")
        .map(|value| value.items)
        .unwrap_or(0);
    let protected = protection
        .iter()
        .find(|value| value.key == "replica_verified")
        .map(|value| value.items)
        .unwrap_or(0);
    let pending = protection
        .iter()
        .filter(|value| value.key != "replica_verified" && value.key != "error")
        .map(|value| value.items)
        .sum();
    let errors = protection
        .iter()
        .find(|value| value.key == "error")
        .map(|value| value.items)
        .unwrap_or(0);
    let formats = extensions
        .iter()
        .map(|value| {
            let descriptor = crate::formats::descriptor(&value.key);
            DashboardFormat {
                key: value.key.clone(),
                label: descriptor.label.into(),
                family: descriptor.family.as_str().into(),
                support: descriptor.support.as_str().into(),
                items: value.items,
                bytes: value.bytes,
            }
        })
        .collect();
    Ok(DashboardStats {
        total_assets,
        photos: total_assets - videos,
        videos,
        bytes,
        protected,
        pending,
        duplicate_groups: 0,
        duplicate_bytes: 0,
        reclaimable_bytes: 0,
        errors,
        offline_sources: 0,
        oldest: years
            .iter()
            .map(|value| value.key.as_str())
            .min()
            .map(|year| format!("{year}-01-01T00:00:00Z")),
        newest: years
            .iter()
            .map(|value| value.key.as_str())
            .max()
            .map(|year| format!("{year}-12-31T23:59:59Z")),
        oldest_photo: None,
        newest_photo: None,
        oldest_video: None,
        newest_video: None,
        master_available_bytes: 0,
        backup_available_bytes: 0,
        types,
        years,
        protection,
        protection_years: Vec::new(),
        protection_sources: Vec::new(),
        sources: Vec::new(),
        months: read("month")?,
        formats,
        cameras: read("camera")?,
        insights: Vec::new(),
        latest_benchmark: None,
        recent_benchmarks: Vec::new(),
        snapshot_generated_at: Utc::now().to_rfc3339(),
        stale: true,
        timings: vec![DashboardTiming {
            section: "snapshot".into(),
            milliseconds: started.elapsed().as_millis() as i64,
        }],
        storage: DashboardStorage::default(),
        technical: DashboardTechnical::default(),
        codecs: Vec::new(),
    })
}

#[tauri::command]
async fn refresh_dashboard(state: State<'_, AppState>) -> Result<DashboardStats, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || {
        let started = std::time::Instant::now();
        let result = compute_dashboard(&cfg)?;
        save_dashboard_snapshot(&cfg, &result, started.elapsed().as_millis() as i64)?;
        Ok(result)
    })
    .await
    .map_err(|error| error.to_string())?
}

fn save_dashboard_snapshot(
    cfg: &LibraryConfig,
    result: &DashboardStats,
    generation_ms: i64,
) -> Result<(), String> {
    let conn = db(cfg)?;
    let payload = serde_json::to_string(result).map_err(|error| error.to_string())?;
    conn.execute("INSERT INTO dashboard_snapshots(id,schema_version,generated_at,invalidated_at,payload,generation_ms,catalog_items)VALUES(1,2,?1,NULL,?2,?3,?4)ON CONFLICT(id)DO UPDATE SET schema_version=excluded.schema_version,generated_at=excluded.generated_at,invalidated_at=NULL,payload=excluded.payload,generation_ms=excluded.generation_ms,catalog_items=excluded.catalog_items",params![result.snapshot_generated_at,payload,generation_ms,result.total_assets]).map_err(|error|error.to_string())?;
    let section = |name: &str| {
        result
            .timings
            .iter()
            .find(|value| value.section == name)
            .map(|value| value.milliseconds)
            .unwrap_or(0)
    };
    conn.execute("INSERT INTO dashboard_metrics(generated_at,mode,total_ms,catalog_ms,rollups_ms,storage_ms,insights_ms,items)VALUES(?1,'full',?2,?3,?4,?5,?6,?7)",params![result.snapshot_generated_at,generation_ms,section("catalog"),section("rollups"),section("storage"),section("insights"),result.total_assets]).map_err(|error|error.to_string())?;
    conn.execute("DELETE FROM dashboard_metrics WHERE id NOT IN(SELECT id FROM dashboard_metrics ORDER BY id DESC LIMIT 100)",[]).map_err(|error|error.to_string())?;
    Ok(())
}

fn compute_dashboard(cfg: &LibraryConfig) -> Result<DashboardStats, String> {
    let total_started = std::time::Instant::now();
    let conn = db(cfg)?;
    let row=conn.query_row("SELECT COUNT(*),COALESCE(SUM(media_type!='video'),0),COALESCE(SUM(media_type='video'),0),COALESCE(SUM(bytes),0),COALESCE(SUM(protection_state='replica_verified'),0),COALESCE(SUM(protection_state IN('source_only','consolidated','stale')),0),COALESCE(SUM(protection_state='error'),0) FROM assets",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).map_err(|e|e.to_string())?;
    let duplicate_groups=conn.query_row("SELECT COUNT(*) FROM(SELECT asset_id FROM active_occurrences GROUP BY asset_id HAVING COUNT(*)>1)",[],|r|r.get(0)).unwrap_or(0);
    let offline = conn
        .query_row(
            "SELECT COUNT(*) FROM sources WHERE available=0 AND path NOT LIKE 'lumina://%'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let rollup = |dimension: &str| -> Result<Vec<DashboardBreakdown>, String> {
        let mut statement=conn.prepare("SELECT key,items,bytes FROM library_rollups WHERE dimension=?1 AND items>0 ORDER BY bytes DESC").map_err(|e|e.to_string())?;
        let values = statement
            .query_map([dimension], |r| {
                Ok(DashboardBreakdown {
                    key: r.get(0)?,
                    items: r.get(1)?,
                    bytes: r.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(values)
    };
    let (oldest, newest): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT MIN(captured_at),MAX(captured_at)FROM assets",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let (oldest_photo, newest_photo, oldest_video, newest_video): (Option<String>, Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT MIN(CASE WHEN media_type IN('photo','raw') THEN captured_at END),MAX(CASE WHEN media_type IN('photo','raw') THEN captured_at END),MIN(CASE WHEN media_type='video' THEN captured_at END),MAX(CASE WHEN media_type='video' THEN captured_at END) FROM assets",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|e| e.to_string())?;
    let sources = {
        let mut statement=conn.prepare("SELECT s.id,s.name,s.available,COUNT(DISTINCT o.asset_id),COALESCE(SUM(a.bytes),0)FROM sources s LEFT JOIN occurrences o ON o.source_id=s.id LEFT JOIN assets a ON a.id=o.asset_id WHERE s.path!='lumina://maintenance' GROUP BY s.id ORDER BY 5 DESC").map_err(|e|e.to_string())?;
        let values = statement
            .query_map([], |r| {
                Ok(DashboardSource {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    available: r.get::<_, i64>(2)? != 0,
                    items: r.get(3)?,
                    bytes: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let pending_bytes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(bytes),0)FROM assets WHERE protection_state!='replica_verified'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let duplicate_bytes:i64=conn.query_row("SELECT COALESCE(SUM(a.bytes*(x.copies-1)),0)FROM assets a JOIN(SELECT asset_id,COUNT(*) copies FROM active_occurrences GROUP BY asset_id HAVING copies>1)x ON x.asset_id=a.id",[],|r|r.get(0)).unwrap_or(0);
    let suspicious:i64=conn.query_row("SELECT COUNT(*)FROM assets WHERE date_source='filesystem_modified' OR CAST(substr(captured_at,1,4)AS INTEGER)<1990 OR CAST(substr(captured_at,1,4)AS INTEGER)>CAST(strftime('%Y','now')AS INTEGER)+1",[],|r|r.get(0)).unwrap_or(0);
    let format_mismatches: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM asset_technical_metadata WHERE extension_matches=0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let technical=conn.query_row("SELECT COUNT(*),COALESCE(SUM(support_level='complete'),0),COALESCE(SUM(support_level='partial'),0),COALESCE(SUM(support_level='preservation'),0),COALESCE(SUM(support_level='unknown'),0),COALESCE(SUM(extension_matches=0),0),COALESCE(SUM(codec IS NOT NULL),0),COALESCE(SUM(family='video' AND codec IS NULL),0),COALESCE(SUM(inventory_state='complete'),0) FROM asset_technical_metadata",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?,r.get::<_,i64>(5)?,r.get::<_,i64>(6)?,r.get::<_,i64>(7)?,r.get::<_,i64>(8)?))).map_err(|e|e.to_string())?;
    let thumbnail_states=conn.query_row("SELECT COALESCE(SUM(state='ready'),0),COALESCE(SUM(state='pending'),0),COALESCE(SUM(state='failed'),0) FROM thumbnails",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?))).unwrap_or((0,0,0));
    let review=conn.query_row("SELECT COUNT(*),COALESCE(SUM(bytes),0) FROM job_items WHERE state='review' AND job_id=(SELECT id FROM jobs WHERE source_id!='_lumina_maintenance' ORDER BY created_at DESC LIMIT 1)",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?))).unwrap_or((0,0));
    let mut insights = Vec::new();
    if pending_bytes > 0 {
        insights.push(DashboardInsight {
            kind: "protection".into(),
            severity: "high".into(),
            title: "Proteção pendente".into(),
            detail: "Mídias consolidadas ainda não possuem réplica local verificada.".into(),
            value: row.5,
            bytes: pending_bytes,
            action: "protection".into(),
            action_label: "Ver proteção".into(),
            confidence: "high".into(),
            reason: "Há itens sem réplica local verificada".into(),
        });
    }
    if duplicate_bytes > 0 {
        insights.push(DashboardInsight {
            kind: "duplicates".into(),
            severity: "medium".into(),
            title: "Conteúdo em várias origens".into(),
            detail: "Espaço ocupado por ocorrências adicionais conhecidas nas fontes.".into(),
            value: duplicate_groups,
            bytes: duplicate_bytes,
            action: "duplicates".into(),
            action_label: "Analisar ocorrências".into(),
            confidence: "high".into(),
            reason: "Estimativa baseada em hashes idênticos e ocorrências conhecidas".into(),
        });
    }
    if suspicious > 0 {
        insights.push(DashboardInsight {
            kind: "dates".into(),
            severity: "low".into(),
            title: "Datas para revisar".into(),
            detail: "Itens usam data do arquivo ou possuem ano suspeito.".into(),
            value: suspicious,
            bytes: 0,
            action: "library".into(),
            action_label: "Revisar datas".into(),
            confidence: "medium".into(),
            reason: "Data de arquivo usada como fallback ou ano fora da faixa esperada".into(),
        });
    }
    if offline > 0 {
        insights.push(DashboardInsight {
            kind: "sources".into(),
            severity: "medium".into(),
            title: "Fontes desconectadas".into(),
            detail: "O inventário foi preservado, mas a origem não está acessível.".into(),
            value: offline,
            bytes: 0,
            action: "sources".into(),
            action_label: "Ver fontes".into(),
            confidence: "high".into(),
            reason: "O volume registrado não está acessível neste momento".into(),
        });
    }
    if format_mismatches > 0 {
        insights.push(DashboardInsight {
            kind: "formats".into(),
            severity: "medium".into(),
            title: "Extensões divergentes".into(),
            detail: "O conteúdo detectado não corresponde ao nome de alguns arquivos.".into(),
            value: format_mismatches,
            bytes: 0,
            action: "library".into(),
            action_label: "Revisar formatos".into(),
            confidence: "high".into(),
            reason: "Assinatura interna comparada com a extensão declarada".into(),
        });
    }
    if thumbnail_states.1 + thumbnail_states.2 > 0 {
        insights.push(DashboardInsight {
            kind: "thumbnails".into(),
            severity: if thumbnail_states.2 > 0 {
                "medium"
            } else {
                "low"
            }
            .into(),
            title: "Galeria sendo preparada".into(),
            detail: format!(
                "{} miniaturas pendentes e {} com falha.",
                thumbnail_states.1, thumbnail_states.2
            ),
            value: thumbnail_states.1 + thumbnail_states.2,
            bytes: 0,
            action: "library".into(),
            action_label: "Acompanhar galeria".into(),
            confidence: "high".into(),
            reason: "Cobertura calculada pelo catálogo de miniaturas".into(),
        });
    }
    let benchmark_sql="SELECT j.id,j.total_items,j.total_bytes,COALESCE((SELECT duration_ms FROM job_metrics WHERE job_id=j.id AND stage='analysis_total'),0),COALESCE((SELECT duration_ms FROM job_metrics WHERE job_id=j.id AND stage='hashing_total'),0),COALESCE((SELECT duration_ms FROM job_metrics WHERE job_id=j.id AND stage='copy_and_verify'),0),COALESCE((SELECT duration_ms FROM job_metrics WHERE job_id=j.id AND stage='thumbnails'),0),COALESCE((SELECT items FROM job_metrics WHERE job_id=j.id AND stage='hashing_workers'),0),COALESCE((SELECT bytes FROM job_metrics WHERE job_id=j.id AND stage='hashing_workers'),0),COALESCE((SELECT items FROM job_metrics WHERE job_id=j.id AND stage='deferred_hash'),0),COALESCE((SELECT items FROM job_metrics WHERE job_id=j.id AND stage='cache_hits'),0)FROM jobs j WHERE EXISTS(SELECT 1 FROM job_metrics m WHERE m.job_id=j.id AND m.stage='analysis_total')ORDER BY j.created_at DESC LIMIT 5";
    let recent_benchmarks = {
        let mut statement = conn.prepare(benchmark_sql).map_err(|e| e.to_string())?;
        let values = statement
            .query_map([], |r| {
                Ok(DashboardBenchmark {
                    job_id: r.get(0)?,
                    items: r.get(1)?,
                    bytes: r.get(2)?,
                    analysis_ms: r.get(3)?,
                    hashing_ms: r.get(4)?,
                    copy_ms: r.get(5)?,
                    thumbnails_ms: r.get(6)?,
                    hash_workers: r.get(7)?,
                    hashed_bytes: r.get(8)?,
                    deferred_hash_items: r.get(9)?,
                    cache_hits: r.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let latest_benchmark = recent_benchmarks.first().cloned();
    let formats = {
        let mut statement=conn.prepare("SELECT LOWER(a.extension),COALESCE(t.family,''),COALESCE(t.support_level,''),COUNT(*),COALESCE(SUM(a.bytes),0) FROM assets a LEFT JOIN asset_technical_metadata t ON t.asset_id=a.id GROUP BY 1,2,3 ORDER BY 5 DESC").map_err(|error|error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        rows.into_iter()
            .map(|(key, family, support, items, bytes)| {
                let descriptor = crate::formats::descriptor(&key);
                DashboardFormat {
                    key,
                    label: descriptor.label.into(),
                    family: if family.is_empty() {
                        descriptor.family.as_str().into()
                    } else {
                        family
                    },
                    support: if support.is_empty() {
                        descriptor.support.as_str().into()
                    } else {
                        support
                    },
                    items,
                    bytes,
                }
            })
            .collect()
    };
    let grouped = |sql: &str| -> Result<Vec<DashboardBreakdown>, String> {
        let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
        let values = statement
            .query_map([], |row| {
                Ok(DashboardBreakdown {
                    key: row.get(0)?,
                    items: row.get(1)?,
                    bytes: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(values)
    };
    let protection_years=grouped("SELECT substr(captured_at,1,4),COUNT(*),COALESCE(SUM(bytes),0) FROM assets WHERE protection_state='replica_verified' GROUP BY 1 ORDER BY 1 DESC")?;
    let protection_sources=grouped("SELECT s.name,COUNT(DISTINCT a.id),COALESCE(SUM(a.bytes),0) FROM sources s JOIN occurrences o ON o.source_id=s.id JOIN assets a ON a.id=o.asset_id WHERE a.protection_state='replica_verified' GROUP BY s.id ORDER BY 3 DESC")?;
    let codecs=grouped("SELECT COALESCE(codec,'Não identificado'),COUNT(*),COALESCE(SUM(a.bytes),0) FROM asset_technical_metadata t JOIN assets a ON a.id=t.asset_id WHERE t.family='video' GROUP BY codec ORDER BY 3 DESC")?;
    let catalog_ms = total_started.elapsed().as_millis() as i64;
    let storage_started = std::time::Instant::now();
    let master_available_bytes = fs2::available_space(&cfg.master_path).unwrap_or(0);
    let backup_available_bytes = fs2::available_space(&cfg.backup_path).unwrap_or(0);
    let master_total_bytes = fs2::total_space(&cfg.master_path).unwrap_or(0);
    let backup_available = Path::new(&cfg.backup_path).exists();
    let backup_total_bytes = fs2::total_space(&cfg.backup_path).unwrap_or(0);
    let cache_bytes: u64 = conn
        .query_row(
            "SELECT COALESCE(SUM(file_bytes),0) FROM thumbnails WHERE state='ready'",
            [],
            |r| r.get::<_, u64>(0),
        )
        .unwrap_or(0);
    let temporary_bytes = 0;
    let average_asset_bytes = if row.0 > 0 { row.3 / row.0 } else { 0 };
    let p90_asset_bytes = if row.0 > 0 {
        conn.query_row(
            "SELECT bytes FROM assets ORDER BY bytes LIMIT 1 OFFSET ?1",
            [(((row.0 - 1) as f64 * 0.9) as i64)],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(average_asset_bytes)
    } else {
        0
    };
    let reserve_bytes = (backup_total_bytes / 20).max(1_073_741_824);
    let projected_backup_free_bytes =
        backup_available_bytes as i64 - pending_bytes - reserve_bytes as i64;
    if pending_bytes > 0 && backup_available {
        insights.push(DashboardInsight {
            kind: "capacity".into(),
            severity: if projected_backup_free_bytes < 0 {
                "high"
            } else {
                "medium"
            }
            .into(),
            title: if projected_backup_free_bytes < 0 {
                "Backup sem capacidade suficiente"
            } else {
                "Backup comporta a réplica atual"
            }
            .into(),
            detail: if projected_backup_free_bytes < 0 {
                format!(
                    "Faltam {} bytes além da reserva.",
                    -projected_backup_free_bytes
                )
            } else {
                format!(
                    "Restarão aproximadamente {} bytes após réplica e reserva.",
                    projected_backup_free_bytes
                )
            },
            value: row.5,
            bytes: pending_bytes,
            action: "protection".into(),
            action_label: "Planejar proteção".into(),
            confidence: "high".into(),
            reason: "Projeção usa espaço livre atual, pendências e 5% de reserva".into(),
        });
    }
    let storage_ms = storage_started.elapsed().as_millis() as i64;
    Ok(DashboardStats {
        total_assets: row.0,
        photos: row.1,
        videos: row.2,
        bytes: row.3,
        protected: row.4,
        pending: row.5,
        duplicate_groups,
        duplicate_bytes,
        reclaimable_bytes: conn.query_row("SELECT COALESCE(SUM(a.bytes*(x.copies-1)),0) FROM assets a JOIN(SELECT asset_id,COUNT(*) copies FROM active_occurrences GROUP BY asset_id HAVING copies>1)x ON x.asset_id=a.id WHERE a.protection_state='replica_verified'",[],|row|row.get(0)).unwrap_or(0),
        errors: row.6,
        offline_sources: offline,
        oldest,
        newest,
        oldest_photo,
        newest_photo,
        oldest_video,
        newest_video,
        master_available_bytes,
        backup_available_bytes,
        types: rollup("type")?,
        years: rollup("year")?,
        protection: rollup("protection")?,
        protection_years,
        protection_sources,
        sources,
        months: grouped("SELECT key,items,bytes FROM library_rollups WHERE dimension='month' AND items>0 ORDER BY key DESC")?,
        formats,
        cameras: rollup("camera")?,
        insights,
        latest_benchmark,
        recent_benchmarks,
        snapshot_generated_at: chrono::Utc::now().to_rfc3339(),
        stale:false,
        timings:vec![
            DashboardTiming{section:"catalog".into(),milliseconds:catalog_ms},
            DashboardTiming{section:"storage".into(),milliseconds:storage_ms},
            DashboardTiming{section:"total".into(),milliseconds:total_started.elapsed().as_millis() as i64}
        ],
        storage:DashboardStorage{master_total_bytes,master_used_bytes:master_total_bytes.saturating_sub(master_available_bytes),master_free_bytes:master_available_bytes,library_bytes:row.3.max(0) as u64,cache_bytes,temporary_bytes,backup_total_bytes,backup_used_bytes:backup_total_bytes.saturating_sub(backup_available_bytes),backup_free_bytes:backup_available_bytes,pending_backup_bytes:pending_bytes.max(0) as u64,projected_backup_free_bytes,reserve_bytes,estimated_additional_items:if average_asset_bytes>0{(master_available_bytes/(average_asset_bytes as u64)) as i64}else{0},average_asset_bytes,p90_asset_bytes,backup_available},
        technical:DashboardTechnical{enriched:technical.0,complete:technical.1,partial:technical.2,preservation:technical.3,unknown:technical.4,mismatches:technical.5,codec_known:technical.6,codec_missing:technical.7,thumbnails_ready:thumbnail_states.0,thumbnails_pending:thumbnail_states.1,thumbnails_failed:thumbnail_states.2,metadata_complete:technical.8,review_items:review.0,review_bytes:review.1},
        codecs,
    })
}
#[tauri::command]
fn list_sources(state: State<AppState>) -> Result<Vec<Source>, String> {
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let mut stmt=conn.prepare("SELECT id,name,COALESCE(mount_path,path),volume_label,last_scan,asset_count FROM sources WHERE path NOT LIKE 'lumina://%' ORDER BY last_scan DESC").map_err(|e|e.to_string())?;
    let raw = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    Ok(raw
        .into_iter()
        .map(|r| {
            let available = Path::new(&r.2).exists();
            conn.execute(
                "UPDATE sources SET available=?2 WHERE id=?1",
                params![r.0, available],
            )
            .ok();
            Source {
                id: r.0,
                name: r.1,
                path: r.2,
                volume_label: r.3,
                available,
                last_scan: r.4,
                asset_count: r.5,
            }
        })
        .collect())
}
#[tauri::command]
fn start_source_sync(
    source_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<String, String> {
    manager.start_source_sync(current(&state)?, source_id)
}
#[tauri::command]
fn get_review_summary(state: State<AppState>) -> Result<ReviewSummary, String> {
    review::summary(&current(&state)?)
}
#[tauri::command]
fn get_library_health(state: State<AppState>) -> Result<LibraryHealth, String> {
    health::inspect(&current(&state)?)
}
#[tauri::command]
fn record_client_error(kind: String, message: String) {
    diagnostics::client_error(&kind, &message);
}
#[tauri::command]
fn undo_last_edit(state: State<AppState>) -> Result<BatchResult, String> {
    review::undo_last(&current(&state)?)
}
#[tauri::command]
fn list_assets(query: String, state: State<AppState>) -> Result<Vec<MediaAsset>, String> {
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let like = format!("%{}%", query);
    let mut stmt=conn.prepare("SELECT a.id,a.filename,a.media_type,a.extension,a.captured_at,a.date_source,a.bytes,a.width,a.height,a.duration,a.camera,a.latitude,a.longitude,a.master_path,a.hash,a.protection_state,(SELECT COUNT(*) FROM occurrences o WHERE o.asset_id=a.id) FROM assets a WHERE a.filename LIKE ?1 OR COALESCE(a.camera,'') LIKE ?1 OR EXISTS(SELECT 1 FROM asset_tags at JOIN tags t ON t.id=at.tag_id WHERE at.asset_id=a.id AND t.name LIKE ?1) ORDER BY a.captured_at DESC LIMIT 5000").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([like], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, Option<i64>>(7)?,
                r.get::<_, Option<i64>>(8)?,
                r.get::<_, Option<f64>>(9)?,
                r.get::<_, Option<String>>(10)?,
                r.get::<_, Option<f64>>(11)?,
                r.get::<_, Option<f64>>(12)?,
                r.get::<_, String>(13)?,
                r.get::<_, String>(14)?,
                r.get::<_, String>(15)?,
                r.get::<_, i64>(16)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    Ok(rows
        .into_iter()
        .map(|r| MediaAsset {
            id: r.0.clone(),
            filename: r.1,
            media_type: r.2,
            extension: r.3,
            captured_at: r.4,
            date_source: r.5,
            date_suspicious: false,
            bytes: r.6,
            width: r.7,
            height: r.8,
            duration: r.9,
            camera: r.10,
            latitude: r.11,
            longitude: r.12,
            thumbnail: None,
            master_path: r.13,
            hash: r.14,
            protection_state: r.15,
            occurrence_count: r.16,
            source_names: engine::grouped_sources(&conn, &r.0),
            tags: engine::grouped_tags(&conn, &r.0),
            favorite: false,
            rating: 0,
            review_later: false,
            description: String::new(),
        })
        .collect())
}
#[tauri::command]
async fn search_gallery(
    request: GalleryRequest,
    state: State<'_, AppState>,
) -> Result<GalleryResult, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || gallery::search(&db(&cfg)?, &request))
        .await
        .map_err(|error| error.to_string())?
}
#[tauri::command]
fn list_duplicates(state: State<AppState>) -> Result<Vec<DuplicateGroup>, String> {
    duplicates::list(&current(&state)?)
}
#[tauri::command]
fn get_duplicate_status(state: State<AppState>) -> Result<DuplicateStatus, String> {
    duplicates::status(&current(&state)?)
}
#[tauri::command]
fn update_duplicate_decision(
    asset_id: String,
    decision: String,
    reason: String,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    duplicates::decide_group(&current(&state)?, &asset_id, &decision, &reason)
}
#[tauri::command]
fn update_occurrence_decision(
    occurrence_id: String,
    decision: String,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    duplicates::decide_occurrence(&current(&state)?, &occurrence_id, &decision)
}
#[tauri::command]
fn create_cleanup_plan(state: State<AppState>) -> Result<CleanupPlan, String> {
    duplicates::create_plan(&current(&state)?)
}
#[tauri::command]
fn export_cleanup_plan(plan_id: String, state: State<AppState>) -> Result<ReportExport, String> {
    duplicates::export_plan(&current(&state)?, &plan_id)
}
#[tauri::command]
fn list_albums(state: State<AppState>) -> Result<Vec<Album>, String> {
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let mut stmt=conn.prepare("SELECT a.id,a.name,COUNT(aa.asset_id) FROM albums a LEFT JOIN album_assets aa ON aa.album_id=a.id GROUP BY a.id ORDER BY a.name").map_err(|e|e.to_string())?;
    let result = stmt
        .query_map([], |r| {
            Ok(Album {
                id: r.get(0)?,
                name: r.get(1)?,
                asset_count: r.get(2)?,
                cover: None,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string());
    result
}
#[tauri::command]
fn list_jobs(state: State<AppState>) -> Result<Vec<JobOverview>, String> {
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let mut stmt=conn.prepare("SELECT j.id,s.name,j.source_path,j.state,j.stage,j.processed_items,j.total_items,j.processed_bytes,j.total_bytes,CASE WHEN j.state='analyzing' AND j.stage_total_bytes>0 THEN MIN(100.0,j.stage_processed_bytes*100.0/j.stage_total_bytes) WHEN j.state='analyzing' AND j.stage_total_items>0 THEN MIN(100.0,j.stage_processed_items*100.0/j.stage_total_items) WHEN j.total_bytes>0 THEN MIN(100.0,j.processed_bytes*100.0/j.total_bytes) WHEN j.total_items>0 THEN MIN(100.0,j.processed_items*100.0/j.total_items) ELSE 0 END,j.bytes_per_second,j.estimated_seconds_remaining,j.imported_count,j.duplicate_count,j.excluded_count,j.failed_count,j.created_at,j.updated_at,j.interruption_reason FROM jobs j JOIN sources s ON s.id=j.source_id ORDER BY CASE WHEN j.state IN('queued','analyzing','consolidating','protecting','pausing','paused','canceling','ready','batch_pending','protection_pending','waiting_space','waiting_backup_space','backup_error','interrupted') THEN 0 ELSE 1 END,j.updated_at DESC LIMIT 200").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(JobOverview {
                job_id: r.get(0)?,
                source_name: r.get(1)?,
                source_path: r.get(2)?,
                state: r.get(3)?,
                stage: r.get(4)?,
                processed_items: r.get(5)?,
                total_items: r.get(6)?,
                processed_bytes: r.get(7)?,
                total_bytes: r.get(8)?,
                overall_percent: r.get(9)?,
                bytes_per_second: r.get(10)?,
                estimated_seconds_remaining: r.get(11)?,
                imported: r.get(12)?,
                duplicates: r.get(13)?,
                excluded: r.get(14)?,
                failed: r.get(15)?,
                created_at: r.get(16)?,
                updated_at: r.get(17)?,
                interruption_reason: r.get(18)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(rows)
}
fn checked_ids(ids: Vec<String>) -> Result<Vec<String>, String> {
    if ids.is_empty() || ids.len() > 5000 {
        return Err("Selecione entre 1 e 5.000 mídias".into());
    }
    Ok(ids)
}
#[tauri::command]
fn create_album(name: String, state: State<AppState>) -> Result<Album, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Nome de álbum inválido".into());
    }
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO albums(id,name,created_at)VALUES(?1,?2,?3)",
        params![id, name, Utc::now().to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;
    Ok(Album {
        id,
        name: name.into(),
        asset_count: 0,
        cover: None,
    })
}
#[tauri::command]
fn rename_album(id: String, name: String, state: State<AppState>) -> Result<BatchResult, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Nome de álbum inválido".into());
    }
    let affected = db(&current(&state)?)?
        .execute("UPDATE albums SET name=?2 WHERE id=?1", params![id, name])
        .map_err(|error| error.to_string())? as i64;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn delete_album(id: String, state: State<AppState>) -> Result<BatchResult, String> {
    let mut conn = db(&current(&state)?)?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM album_assets WHERE album_id=?1", [&id])
        .map_err(|error| error.to_string())?;
    let affected = tx
        .execute("DELETE FROM albums WHERE id=?1", [id])
        .map_err(|error| error.to_string())? as i64;
    tx.commit().map_err(|error| error.to_string())?;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn add_assets_to_album(
    album_id: String,
    asset_ids: Vec<String>,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    let ids = checked_ids(asset_ids)?;
    let cfg = current(&state)?;
    let mut conn = db(&cfg)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    if tx
        .query_row(
            "SELECT COUNT(*) FROM albums WHERE id=?1",
            [&album_id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap_or(0)
        != 1
    {
        return Err("Álbum não encontrado".into());
    }
    let mut affected = 0;
    for id in ids {
        affected+=tx.execute("INSERT OR IGNORE INTO album_assets(album_id,asset_id)SELECT ?1,id FROM assets WHERE id=?2",params![album_id,id]).map_err(|e|e.to_string())? as i64
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn apply_tag(
    tag_name: String,
    asset_ids: Vec<String>,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    let ids = checked_ids(asset_ids)?;
    let name = tag_name.trim().to_lowercase();
    if name.is_empty() || name.chars().count() > 60 {
        return Err("Tag inválida".into());
    }
    let cfg = current(&state)?;
    let mut conn = db(&cfg)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT OR IGNORE INTO tags(id,name)VALUES(?1,?2)",
        params![id, name],
    )
    .map_err(|e| e.to_string())?;
    let tag_id: String = tx
        .query_row("SELECT id FROM tags WHERE name=?1", [&name], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let mut affected = 0;
    for asset in ids {
        affected+=tx.execute("INSERT OR IGNORE INTO asset_tags(asset_id,tag_id)SELECT id,?2 FROM assets WHERE id=?1",params![asset,tag_id]).map_err(|e|e.to_string())? as i64
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn list_tags(state: State<AppState>) -> Result<Vec<TagInfo>, String> {
    let conn = db(&current(&state)?)?;
    let mut statement=conn.prepare("SELECT t.id,t.name,COUNT(at.asset_id)FROM tags t LEFT JOIN asset_tags at ON at.tag_id=t.id GROUP BY t.id ORDER BY LOWER(t.name)").map_err(|error|error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(TagInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                asset_count: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}
#[tauri::command]
fn rename_tag(id: String, name: String, state: State<AppState>) -> Result<BatchResult, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 {
        return Err("Nome de tag inválido".into());
    }
    let conn = db(&current(&state)?)?;
    let affected = conn
        .execute("UPDATE tags SET name=?2 WHERE id=?1", params![id, name])
        .map_err(|error| error.to_string())? as i64;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn delete_tag(id: String, state: State<AppState>) -> Result<BatchResult, String> {
    let mut conn = db(&current(&state)?)?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    tx.execute("DELETE FROM asset_tags WHERE tag_id=?1", [&id])
        .map_err(|error| error.to_string())?;
    let affected = tx
        .execute("DELETE FROM tags WHERE id=?1", [id])
        .map_err(|error| error.to_string())? as i64;
    tx.commit().map_err(|error| error.to_string())?;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn update_capture_date(
    asset_ids: Vec<String>,
    captured_at: String,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    let ids = checked_ids(asset_ids)?;
    let parsed = chrono::DateTime::parse_from_rfc3339(&captured_at)
        .map_err(|_| "Data inválida; use data e hora completas".to_string())?
        .to_rfc3339();
    let cfg = current(&state)?;
    let mut conn = db(&cfg)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut affected = 0;
    for id in ids {
        let old: Option<String> = tx
            .query_row("SELECT captured_at FROM assets WHERE id=?1", [&id], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(old) = old {
            tx.execute("INSERT INTO asset_edits(asset_id,field,old_value,new_value,edited_at)VALUES(?1,'captured_at',?2,?3,?4)",params![id,old,parsed,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
            affected += tx
                .execute(
                    "UPDATE assets SET captured_at=?2,date_source='user_corrected' WHERE id=?1",
                    params![id, parsed],
                )
                .map_err(|e| e.to_string())? as i64
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn update_user_state(
    request: UserStateUpdate,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    let ids = checked_ids(request.asset_ids)?;
    if let Some(rating) = request.rating {
        if !(0..=5).contains(&rating) {
            return Err("A avaliação deve estar entre 0 e 5".into());
        }
    }
    if request
        .description
        .as_ref()
        .is_some_and(|value| value.chars().count() > 2000)
    {
        return Err("A descrição deve ter no máximo 2.000 caracteres".into());
    }
    let cfg = current(&state)?;
    let mut conn = db(&cfg)?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let now = Utc::now().to_rfc3339();
    let mut affected = 0;
    for asset in ids {
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM assets WHERE id=?1)",
                [&asset],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if !exists {
            continue;
        }
        let old: (bool,i64,bool,String) = tx.query_row("SELECT favorite,rating,review_later,description FROM asset_user_state WHERE asset_id=?1",[&asset],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).optional().map_err(|error|error.to_string())?.unwrap_or((false,0,false,String::new()));
        let next = (
            request.favorite.unwrap_or(old.0),
            request.rating.unwrap_or(old.1),
            request.review_later.unwrap_or(old.2),
            request.description.clone().unwrap_or(old.3.clone()),
        );
        tx.execute("INSERT INTO asset_user_state(asset_id,favorite,rating,review_later,description,updated_at)VALUES(?1,?2,?3,?4,?5,?6)ON CONFLICT(asset_id)DO UPDATE SET favorite=excluded.favorite,rating=excluded.rating,review_later=excluded.review_later,description=excluded.description,updated_at=excluded.updated_at",params![asset,next.0,next.1,next.2,next.3,now]).map_err(|error|error.to_string())?;
        tx.execute("INSERT INTO asset_edits(asset_id,field,old_value,new_value,edited_at)VALUES(?1,'user_state',?2,?3,?4)",params![asset,serde_json::to_string(&old).unwrap_or_default(),serde_json::to_string(&next).unwrap_or_default(),now]).map_err(|error|error.to_string())?;
        affected += 1;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(BatchResult { affected })
}

#[tauri::command]
fn list_saved_views(state: State<AppState>) -> Result<Vec<SavedView>, String> {
    let conn = db(&current(&state)?)?;
    let mut statement=conn.prepare("SELECT id,name,filters_json,smart_album,created_at,updated_at FROM saved_views ORDER BY smart_album DESC,name").map_err(|error|error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .map(|row| {
            let (id, name, json, smart_album, created_at, updated_at) =
                row.map_err(|error| error.to_string())?;
            let filters = serde_json::from_str(&json).map_err(|error| error.to_string())?;
            Ok(SavedView {
                id,
                name,
                filters,
                smart_album,
                created_at,
                updated_at,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(rows)
}

#[tauri::command]
fn save_gallery_view(
    name: String,
    filters: GalleryFilters,
    smart_album: bool,
    state: State<AppState>,
) -> Result<SavedView, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Nome de visão inválido".into());
    }
    let conn = db(&current(&state)?)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let json = serde_json::to_string(&filters).map_err(|error| error.to_string())?;
    conn.execute("INSERT INTO saved_views(id,name,filters_json,smart_album,created_at,updated_at)VALUES(?1,?2,?3,?4,?5,?5)ON CONFLICT(name)DO UPDATE SET filters_json=excluded.filters_json,smart_album=excluded.smart_album,updated_at=excluded.updated_at",params![id,name,json,smart_album,now]).map_err(|error|error.to_string())?;
    let actual_id = conn
        .query_row("SELECT id FROM saved_views WHERE name=?1", [name], |row| {
            row.get(0)
        })
        .map_err(|error| error.to_string())?;
    Ok(SavedView {
        id: actual_id,
        name: name.into(),
        filters,
        smart_album,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
fn delete_saved_view(id: String, state: State<AppState>) -> Result<BatchResult, String> {
    let conn = db(&current(&state)?)?;
    let affected = conn
        .execute("DELETE FROM saved_views WHERE id=?1", [id])
        .map_err(|error| error.to_string())? as i64;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn rename_saved_view(
    id: String,
    name: String,
    state: State<AppState>,
) -> Result<BatchResult, String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("Nome de visão inválido".into());
    }
    let affected = db(&current(&state)?)?
        .execute(
            "UPDATE saved_views SET name=?2,updated_at=?3 WHERE id=?1",
            params![id, name, Utc::now().to_rfc3339()],
        )
        .map_err(|error| error.to_string())? as i64;
    Ok(BatchResult { affected })
}
#[tauri::command]
fn list_events(state: State<AppState>) -> Result<Vec<ImportEvent>, String> {
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let mut stmt = conn
        .prepare("SELECT id,job_id,at,path,state,details FROM events ORDER BY id DESC LIMIT 1000")
        .map_err(|e| e.to_string())?;
    let result = stmt
        .query_map([], |r| {
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
        .map_err(|e| e.to_string());
    result
}
#[tauri::command]
async fn analyze_source(
    source_path: String,
    source_name: String,
    state: State<'_, AppState>,
) -> Result<ImportSummary, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || engine::analyze(&cfg, &source_path, &source_name))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn consolidate_import(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || engine::consolidate(&cfg, &job_id))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn get_job_progress(job_id: String, state: State<AppState>) -> Result<JobProgress, String> {
    let cfg = current(&state)?;
    engine::job_progress(&cfg, &job_id)
}
#[tauri::command]
fn get_import_summary(job_id: String, state: State<AppState>) -> Result<ImportSummary, String> {
    engine::import_summary(&current(&state)?, &job_id)
}
#[tauri::command]
fn get_storage_plan(job_id: String, state: State<AppState>) -> Result<StoragePlan, String> {
    engine::storage_plan(&current(&state)?, &job_id)
}
#[tauri::command]
fn update_job_selection(
    request: SelectionRequest,
    state: State<AppState>,
) -> Result<SelectionResult, String> {
    engine::apply_selection(&current(&state)?, &request)
}
#[tauri::command]
fn get_protection_queue(
    job_id: Option<String>,
    state: State<AppState>,
) -> Result<ProtectionQueueStats, String> {
    engine::protection_stats(&current(&state)?, job_id.as_deref())
}
#[tauri::command]
fn control_import(
    job_id: String,
    action: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<JobProgress, String> {
    let cfg = current(&state)?;
    if action == "canceled" {
        manager.cancel(&job_id);
    }
    engine::set_job_state(&cfg, &job_id, &action)
}
#[tauri::command]
fn pause_job(job_id: String, state: State<AppState>) -> Result<JobProgress, String> {
    engine::set_job_state(&current(&state)?, &job_id, "paused")
}
#[tauri::command]
fn cancel_job(
    job_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<JobProgress, String> {
    manager.cancel(&job_id);
    engine::set_job_state(&current(&state)?, &job_id, "canceled")
}
#[tauri::command]
fn get_job_snapshot(job_id: String, state: State<AppState>) -> Result<JobProgress, String> {
    engine::job_progress(&current(&state)?, &job_id)
}
#[tauri::command]
fn verify_backup(
    state: State<AppState>,
    manager: State<jobs::JobManager>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let cfg = current(&state)?;
    let job = manager.start_verification(cfg.clone())?;
    jobs::emit_progress(app, cfg, job.clone());
    Ok(job)
}
#[tauri::command]
fn start_analysis(
    source_path: String,
    source_name: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let cfg = current(&state)?;
    let job = manager.start_analysis(cfg.clone(), source_path, source_name)?;
    jobs::emit_progress(app, cfg, job.clone());
    Ok(job)
}
#[tauri::command]
fn start_format_enrichment(
    state: State<AppState>,
    manager: State<jobs::JobManager>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let cfg = current(&state)?;
    let job = manager.start_format_enrichment(cfg.clone())?;
    jobs::emit_progress(app, cfg, job.clone());
    Ok(job)
}
#[tauri::command]
fn start_consolidation(
    job_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let cfg = current(&state)?;
    manager.start_consolidation(cfg.clone(), job_id.clone())?;
    jobs::emit_progress(app, cfg, job_id);
    Ok(())
}
#[tauri::command]
fn start_protection(
    job_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let cfg = current(&state)?;
    manager.start_protection(cfg.clone(), job_id.clone())?;
    jobs::emit_progress(app, cfg, job_id);
    Ok(())
}
#[tauri::command]
fn list_recoverable_jobs(state: State<AppState>) -> Result<Vec<RecoverableJob>, String> {
    jobs::JobManager::recoverable(&current(&state)?)
}
#[tauri::command]
fn discard_job(job_id: String, state: State<AppState>) -> Result<(), String> {
    jobs::JobManager::discard(&current(&state)?, &job_id)
}
#[tauri::command]
fn resume_job(
    job_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<(), String> {
    manager.resume(current(&state)?, job_id)
}
#[tauri::command]
fn retry_failed_items(
    job_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<i64, String> {
    let cfg = current(&state)?;
    let changed = manager.retry_failed(&cfg, &job_id)?;
    if changed > 0 {
        manager.resume(cfg, job_id)?;
    }
    Ok(changed)
}
#[tauri::command]
fn get_job_events(
    job_id: String,
    cursor: i64,
    filter: String,
    state: State<AppState>,
) -> Result<JobEventPage, String> {
    events::page(&current(&state)?, &job_id, cursor, &filter)
}
#[tauri::command]
fn export_job_report(
    job_id: String,
    format: String,
    state: State<AppState>,
) -> Result<ReportExport, String> {
    events::export(&current(&state)?, &job_id, &format)
}
#[tauri::command]
fn export_diagnostics(state: State<AppState>) -> Result<ReportExport, String> {
    events::export_diagnostics(&current(&state)?)
}
#[tauri::command]
fn get_thumbnail(
    asset_id: String,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<Option<String>, String> {
    let cfg = current(&state)?;
    let thumbnail = media::thumbnail_file(&cfg, &asset_id)?;
    if thumbnail.is_none() {
        manager.request_thumbnail(cfg, asset_id.clone(), 200)?;
    }
    Ok(thumbnail.map(|_| {
        #[cfg(windows)]
        {
            format!("http://lumina-thumb.localhost/{asset_id}")
        }
        #[cfg(not(windows))]
        {
            format!("lumina-thumb://localhost/{asset_id}")
        }
    }))
}
#[tauri::command]
fn prefetch_thumbnails(
    asset_ids: Vec<String>,
    priority: i64,
    state: State<AppState>,
    manager: State<jobs::JobManager>,
) -> Result<i64, String> {
    let cfg = current(&state)?;
    let mut accepted = 0;
    for asset in asset_ids.into_iter().take(80) {
        if valid_thumbnail_asset_id(&asset) {
            manager.request_thumbnail(cfg.clone(), asset, priority.clamp(10, 180))?;
            accepted += 1;
        }
    }
    Ok(accepted)
}

#[tauri::command]
fn reveal_asset_in_folder(asset_id: String, state: State<AppState>) -> Result<(), String> {
    if !valid_thumbnail_asset_id(&asset_id) {
        return Err("Identificador inválido".into());
    }
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let path: String = conn
        .query_row(
            "SELECT master_path FROM assets WHERE id=?1",
            [&asset_id],
            |row| row.get(0),
        )
        .map_err(|_| "Mídia não encontrada".to_string())?;
    if !std::path::Path::new(&path).is_file() {
        return Err("O arquivo não está disponível no acervo mestre".into());
    }
    #[cfg(windows)]
    std::process::Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn()
        .map_err(|error| error.to_string())?;
    #[cfg(not(windows))]
    return Err("Abrir localização ainda não é suportado neste sistema".into());
    Ok(())
}
#[tauri::command]
fn get_media_url(asset_id: String, state: State<AppState>) -> Result<String, String> {
    if !valid_thumbnail_asset_id(&asset_id) {
        return Err("Identificador inválido".into());
    }
    let cfg = current(&state)?;
    let conn = db(&cfg)?;
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM assets WHERE id=?1)",
            [&asset_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err("Mídia não encontrada".into());
    }
    #[cfg(windows)]
    {
        Ok(format!("http://lumina-media.localhost/{asset_id}"))
    }
    #[cfg(not(windows))]
    {
        Ok(format!("lumina-media://localhost/{asset_id}"))
    }
}

#[tauri::command]
async fn prepare_photo_preview(
    asset_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if !valid_thumbnail_asset_id(&asset_id) {
        return Err("Identificador inválido".into());
    }
    let cfg = current(&state)?;
    let requested = asset_id.clone();
    tauri::async_runtime::spawn_blocking(move || media::viewer_preview_file(&cfg, &requested))
        .await
        .map_err(|error| error.to_string())??;
    #[cfg(windows)]
    {
        Ok(format!("http://lumina-preview.localhost/{asset_id}"))
    }
    #[cfg(not(windows))]
    {
        Ok(format!("lumina-preview://localhost/{asset_id}"))
    }
}

#[tauri::command]
async fn get_asset_details(
    asset_id: String,
    state: State<'_, AppState>,
) -> Result<AssetDetails, String> {
    if !valid_thumbnail_asset_id(&asset_id) {
        return Err("Identificador inválido".into());
    }
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || metadata::details(&cfg, &asset_id))
        .await
        .map_err(|error| error.to_string())?
}

fn valid_thumbnail_asset_id(asset: &str) -> bool {
    !asset.is_empty()
        && asset
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || value == '-')
}
#[tauri::command]
async fn rebuild_thumbnail_cache(state: State<'_, AppState>) -> Result<CacheResult, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || media::rebuild_cache(&cfg))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn audit_thumbnail_cache(
    repair: bool,
    state: State<'_, AppState>,
) -> Result<ThumbnailAudit, String> {
    let cfg = current(&state)?;
    tauri::async_runtime::spawn_blocking(move || media::audit_thumbnails(&cfg, repair))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
fn get_thumbnail_repair_progress() -> ThumbnailRepairProgress {
    media::repair_progress()
}
#[tauri::command]
fn clear_thumbnail_cache(state: State<AppState>) -> Result<i64, String> {
    media::clear_cache(&current(&state)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    diagnostics::start_session();
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_path = base.join("Lumina/library.json");
    let config: Option<LibraryConfig> = fs::read(&config_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok());
    let manager = jobs::JobManager::new();
    let library_lock = config.as_ref().and_then(|cfg| {
        library::LibraryLock::acquire(Path::new(&cfg.master_path), manager.instance_id()).ok()
    });
    if let Some(cfg) = config.as_ref() {
        let _ = jobs::JobManager::interrupt_running(cfg);
        let _ = manager.resume_background(cfg.clone());
    }
    tauri::Builder::default()
        .register_uri_scheme_protocol("lumina-thumb", |context, request| {
            let asset = request.uri().path().trim_start_matches('/');
            let response = || -> Result<Vec<u8>, String> {
                if !valid_thumbnail_asset_id(asset) {
                    return Err("Identificador inválido".into());
                }
                let state = context.app_handle().state::<AppState>();
                let cfg = state
                    .library
                    .lock()
                    .map_err(|_| "Estado indisponível".to_string())?
                    .clone()
                    .ok_or_else(|| "Biblioteca não configurada".to_string())?;
                let path = media::thumbnail_file(&cfg, asset)?
                    .ok_or_else(|| "Miniatura indisponível".to_string())?;
                fs::read(path).map_err(|error| error.to_string())
            };
            match response() {
                Ok(bytes) => tauri::http::Response::builder()
                    .status(200)
                    .header("Content-Type", "image/jpeg")
                    .header("Cache-Control", "public, max-age=31536000, immutable")
                    .body(bytes)
                    .unwrap(),
                Err(error) => tauri::http::Response::builder()
                    .status(404)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(error.into_bytes())
                    .unwrap(),
            }
        })
        .register_uri_scheme_protocol("lumina-preview", |context, request| {
            let asset = request.uri().path().trim_start_matches('/');
            let response = || -> Result<Vec<u8>, String> {
                if !valid_thumbnail_asset_id(asset) {
                    return Err("Identificador inválido".into());
                }
                let state = context.app_handle().state::<AppState>();
                let cfg = state
                    .library
                    .lock()
                    .map_err(|_| "Estado indisponível".to_string())?
                    .clone()
                    .ok_or_else(|| "Biblioteca não configurada".to_string())?;
                fs::read(media::viewer_preview_file(&cfg, asset)?)
                    .map_err(|error| error.to_string())
            };
            match response() {
                Ok(bytes) => tauri::http::Response::builder()
                    .status(200)
                    .header("Content-Type", "image/jpeg")
                    .header("Cache-Control", "private, max-age=86400")
                    .body(bytes)
                    .unwrap(),
                Err(error) => tauri::http::Response::builder()
                    .status(404)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(error.into_bytes())
                    .unwrap(),
            }
        })
        .register_uri_scheme_protocol("lumina-media", |context, request| {
            let asset = request.uri().path().trim_start_matches('/');
            let response = || -> Result<(Vec<u8>, String, u64, u64, u64), String> {
                if !valid_thumbnail_asset_id(asset) {
                    return Err("Identificador inválido".into());
                }
                let state = context.app_handle().state::<AppState>();
                let cfg = state
                    .library
                    .lock()
                    .map_err(|_| "Estado indisponível".to_string())?
                    .clone()
                    .ok_or_else(|| "Biblioteca não configurada".to_string())?;
                let conn = db(&cfg)?;
                let (stored, extension, media_type): (String, String, String) = conn
                    .query_row(
                        "SELECT master_path,LOWER(extension),media_type FROM assets WHERE id=?1",
                        [asset],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .map_err(|_| "Mídia não encontrada".to_string())?;
                // Navegar rapidamente por originais grandes pode esgotar a memória do
                // WebView. Fotos usam a representação limitada do cache; vídeos mantêm
                // acesso parcial ao arquivo original.
                let served = if media_type == "photo" {
                    media::thumbnail_file(&cfg, asset)?
                        .ok_or_else(|| "Prévia indisponível".to_string())?
                } else {
                    PathBuf::from(&stored)
                };
                let canonical = fs::canonicalize(&served).map_err(|error| error.to_string())?;
                let master =
                    fs::canonicalize(&cfg.master_path).map_err(|error| error.to_string())?;
                if !canonical.starts_with(master) {
                    return Err("Mídia fora do acervo".into());
                }
                let mut file = fs::File::open(canonical).map_err(|error| error.to_string())?;
                let total = file.metadata().map_err(|error| error.to_string())?.len();
                let range = request
                    .headers()
                    .get("range")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.strip_prefix("bytes="))
                    .and_then(|value| value.split_once('-'))
                    .map(|(start, end)| {
                        (start.parse::<u64>().unwrap_or(0), end.parse::<u64>().ok())
                    });
                let (start, end) = range
                    .map(|(start, end)| {
                        (
                            start,
                            end.unwrap_or_else(|| {
                                (start + 4 * 1024 * 1024 - 1).min(total.saturating_sub(1))
                            }),
                        )
                    })
                    .unwrap_or((0, total.saturating_sub(1)));
                let end = end.min(total.saturating_sub(1));
                let length = end.saturating_sub(start) + 1;
                file.seek(SeekFrom::Start(start))
                    .map_err(|error| error.to_string())?;
                let mut bytes = vec![0; length as usize];
                file.read_exact(&mut bytes)
                    .map_err(|error| error.to_string())?;
                let mime = if media_type == "photo" {
                    "image/jpeg"
                } else {
                    match extension.as_str() {
                        "mp4" | "m4v" => "video/mp4",
                        "mov" => "video/quicktime",
                        "webm" => "video/webm",
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        _ => "application/octet-stream",
                    }
                }
                .to_string();
                Ok((bytes, mime, start, end, total))
            };
            match response() {
                Ok((bytes, mime, start, end, total)) => {
                    let partial = start > 0 || end + 1 < total;
                    let mut builder = tauri::http::Response::builder()
                        .status(if partial { 206 } else { 200 })
                        .header("Content-Type", mime)
                        .header("Accept-Ranges", "bytes")
                        .header("Content-Length", bytes.len().to_string())
                        .header("Cache-Control", "no-store");
                    if partial {
                        builder =
                            builder.header("Content-Range", format!("bytes {start}-{end}/{total}"));
                    }
                    builder.body(bytes).unwrap()
                }
                Err(error) => tauri::http::Response::builder()
                    .status(404)
                    .header("Content-Type", "text/plain; charset=utf-8")
                    .body(error.into_bytes())
                    .unwrap(),
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(manager)
        .manage(AppState {
            library: Mutex::new(config),
            config_path,
            library_lock: Mutex::new(library_lock),
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            update_backup_path,
            migrate_master_path,
            frontend_ready,
            create_library,
            get_dashboard,
            refresh_dashboard,
            list_sources,
            start_source_sync,
            get_review_summary,
            get_library_health,
            record_client_error,
            undo_last_edit,
            list_assets,
            search_gallery,
            list_duplicates,
            get_duplicate_status,
            update_duplicate_decision,
            update_occurrence_decision,
            create_cleanup_plan,
            export_cleanup_plan,
            list_albums,
            list_jobs,
            create_album,
            rename_album,
            delete_album,
            add_assets_to_album,
            apply_tag,
            list_tags,
            rename_tag,
            delete_tag,
            update_capture_date,
            update_user_state,
            list_saved_views,
            save_gallery_view,
            delete_saved_view,
            rename_saved_view,
            list_events,
            analyze_source,
            consolidate_import,
            get_job_progress,
            get_import_summary,
            get_storage_plan,
            update_job_selection,
            get_protection_queue,
            control_import,
            pause_job,
            cancel_job,
            get_job_snapshot,
            verify_backup,
            start_analysis,
            start_format_enrichment,
            start_consolidation,
            start_protection,
            list_recoverable_jobs,
            discard_job,
            resume_job,
            retry_failed_items,
            get_job_events,
            export_job_report,
            export_diagnostics,
            get_thumbnail,
            prefetch_thumbnails,
            get_media_url,
            reveal_asset_in_folder,
            prepare_photo_preview,
            get_asset_details,
            rebuild_thumbnail_cache,
            audit_thumbnail_cache,
            get_thumbnail_repair_progress,
            clear_thumbnail_cache
        ])
        .build(tauri::generate_context!())
        .expect("erro ao iniciar Lumina")
        .run(|_, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                diagnostics::finish_session();
            }
        });
}

#[cfg(test)]
mod protocol_tests {
    use super::{compute_dashboard, quick_dashboard, valid_thumbnail_asset_id};
    use crate::{catalog, models::LibraryConfig};
    use std::{
        fs,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::Instant,
    };

    #[test]
    fn thumbnail_protocol_rejects_paths_and_accepts_catalog_ids() {
        assert!(valid_thumbnail_asset_id("7c01d129-281e-4da2-a219-f6e704da"));
        assert!(!valid_thumbnail_asset_id("../catalog.sqlite"));
        assert!(!valid_thumbnail_asset_id("folder/asset"));
        assert!(!valid_thumbnail_asset_id(""));
    }

    #[test]
    #[ignore = "benchmark de escala executado na validação de release"]
    fn dashboard_rollups_scale_to_100k_and_500k_with_concurrent_reads() {
        for (items, ceiling_ms) in [(100_000i64, 100u128), (500_000, 300)] {
            let root = std::env::temp_dir()
                .join(format!("lumina-dashboard-{items}-{}", uuid::Uuid::new_v4()));
            let master = root.join("master");
            let backup = root.join("backup");
            fs::create_dir_all(&master).unwrap();
            fs::create_dir_all(&backup).unwrap();
            let cfg = LibraryConfig {
                id: "bench".into(),
                name: "bench".into(),
                master_path: master.to_string_lossy().into(),
                backup_path: backup.to_string_lossy().into(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
            conn.execute_batch("PRAGMA synchronous=OFF; DROP TRIGGER rollup_asset_insert; DROP TRIGGER assets_fts_insert;").unwrap();
            conn.execute("WITH RECURSIVE n(x) AS(SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<?1) INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,protection_state,created_at) SELECT printf('a-%d',x),printf('%064x',x),printf('IMG_%d.jpg',x),CASE WHEN x%10=0 THEN 'video' ELSE 'photo' END,CASE WHEN x%10=0 THEN 'mp4' ELSE 'jpg' END,printf('%04d-01-01T00:00:00Z',2000+x%25),'benchmark',1000000,printf('master/%d',x),CASE WHEN x%3=0 THEN 'replica_verified' ELSE 'consolidated' END,datetime('now') FROM n",[items]).unwrap();
            conn.execute_batch("DELETE FROM library_rollups; INSERT INTO library_rollups SELECT 'type',media_type,COUNT(*),SUM(bytes)FROM assets GROUP BY media_type;INSERT INTO library_rollups SELECT 'year',substr(captured_at,1,4),COUNT(*),SUM(bytes)FROM assets GROUP BY 2;INSERT INTO library_rollups SELECT 'protection',protection_state,COUNT(*),SUM(bytes)FROM assets GROUP BY protection_state;INSERT INTO library_rollups SELECT 'extension',extension,COUNT(*),SUM(bytes)FROM assets GROUP BY extension;INSERT INTO library_rollups SELECT 'month',substr(captured_at,1,7),COUNT(*),SUM(bytes)FROM assets GROUP BY 2;").unwrap();
            drop(conn);
            let running = Arc::new(AtomicBool::new(true));
            let worker_running = running.clone();
            let db_path = master.join(".lumina/catalog.sqlite");
            let reader = std::thread::spawn(move || {
                let c = rusqlite::Connection::open(db_path).unwrap();
                while worker_running.load(Ordering::Relaxed) {
                    let _: i64 = c
                        .query_row(
                            "SELECT COUNT(*) FROM assets WHERE captured_at>'2020'",
                            [],
                            |r| r.get(0),
                        )
                        .unwrap();
                }
            });
            let mut measurements = Vec::new();
            for _ in 0..20 {
                let start = Instant::now();
                let result = quick_dashboard(&cfg).unwrap();
                assert_eq!(result.total_assets, items);
                measurements.push(start.elapsed().as_millis());
            }
            running.store(false, Ordering::Relaxed);
            reader.join().unwrap();
            measurements.sort_unstable();
            let p95 = measurements[18];
            eprintln!("BENCHMARK dashboard_records={items} p50_ms={} p95_ms={p95} concurrent_full_scan=true",measurements[10]);
            assert!(
                p95 <= ceiling_ms,
                "{items} itens: p95 {p95} ms > {ceiling_ms} ms"
            );
            let full_started = Instant::now();
            let full = compute_dashboard(&cfg).unwrap();
            let full_ms = full_started.elapsed().as_millis();
            assert_eq!(full.total_assets, items);
            eprintln!("BENCHMARK dashboard_full_records={items} total_ms={full_ms}");
            assert!(
                full_ms <= if items == 100_000 { 1_500 } else { 4_000 },
                "dashboard completo com {items} itens: {full_ms} ms"
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn dashboard_reports_persisted_storage_and_technical_health() {
        let root =
            std::env::temp_dir().join(format!("lumina-dashboard-health-{}", uuid::Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(&master).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "health".into(),
            name: "health".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('video',?1,'clip.mp4','video','mp4','2025-08-01','metadata',1000,'clip.mp4',datetime('now'))",["a".repeat(64)]).unwrap();
        conn.execute("INSERT INTO asset_technical_metadata(asset_id,declared_extension,detected_format,family,container,codec,support_level,inventory_state,enriched_at)VALUES('video','mp4','video','video','mov,mp4','h264','partial','complete',datetime('now'))",[]).unwrap();
        conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,file_bytes,state,updated_at)VALUES('video',2,'thumb.jpg',321,'ready',datetime('now'))",[]).unwrap();
        drop(conn);
        let result = compute_dashboard(&cfg).unwrap();
        assert_eq!(result.storage.library_bytes, 1000);
        assert_eq!(result.storage.cache_bytes, 321);
        assert_eq!(result.technical.metadata_complete, 1);
        assert_eq!(result.technical.codec_known, 1);
        assert_eq!(result.codecs[0].key, "h264");
        fs::remove_dir_all(root).unwrap();
    }
}
