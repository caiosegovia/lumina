use crate::{catalog, models::*};
use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Instant,
};
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

const MEDIA: &[&str] = crate::pipeline::MEDIA;
type CapturedMetadata = (
    String,
    String,
    Option<i64>,
    Option<i64>,
    Option<f64>,
    Option<String>,
    Option<f64>,
    Option<f64>,
);
type BatchedMetadata = (
    HashMap<String, CapturedMetadata>,
    HashMap<String, crate::media::ValidationResult>,
);
fn progress_due(done: i64, total: i64) -> bool {
    done % 8 == 0 || done == total
}
fn storage_profile(path: &Path, total_bytes: u64, average: u64) -> (&'static str, usize) {
    if let Some(value) = std::env::var("LUMINA_HASH_WORKERS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return ("manual", value.clamp(1, 8));
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::GetDriveTypeW;
        let root = path
            .components()
            .next()
            .map(|x| format!("{}\\", x.as_os_str().to_string_lossy()))
            .unwrap_or_default();
        let wide: Vec<u16> = std::ffi::OsStr::new(&root)
            .encode_wide()
            .chain(Some(0))
            .collect();
        let kind = unsafe { GetDriveTypeW(wide.as_ptr()) };
        if kind == 2 {
            return ("removable", 1);
        }
    }
    if total_bytes > 8 * 1024 * 1024 * 1024 && average > 8 * 1024 * 1024 {
        ("large-media-conservative", 1)
    } else {
        (
            "parallel",
            std::thread::available_parallelism()
                .map(|x| x.get())
                .unwrap_or(2)
                .clamp(2, 4),
        )
    }
}
fn hash_files_adaptive(
    paths: Vec<PathBuf>,
    catalog_path: &Path,
    job: &str,
    total_bytes: u64,
    cancel: &crate::process::CancellationToken,
) -> (HashMap<String, Result<String, String>>, usize, String) {
    if paths.is_empty() {
        return (HashMap::new(), 0, "deferred".into());
    }
    let average = total_bytes / (paths.len() as u64).max(1);
    let (profile, automatic) = storage_profile(&paths[0], total_bytes, average);
    let workers = automatic.min(paths.len());
    let paths = Arc::new(paths);
    let next = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    let bytes = AtomicU64::new(0);
    let results = Mutex::new(HashMap::new());
    let started = Instant::now();
    std::thread::scope(|scope| {
        for _ in 0..workers {
            let paths = &paths;
            let next = &next;
            let completed = &completed;
            let bytes = &bytes;
            let results = &results;
            scope.spawn(move || loop {
                if cancel.is_cancelled() { break; }
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= paths.len() { break; }
                let path = &paths[index];
                let _io = crate::resource::io(crate::resource::Priority::Background);
                let value = crate::storage::sha256_cancel(path, Some(cancel));
                let length = fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0);
                let complete_bytes = bytes.fetch_add(length, Ordering::Relaxed) + length;
                let complete_items = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if progress_due(complete_items as i64, paths.len() as i64) {
                    let speed = complete_bytes as f64 / started.elapsed().as_secs_f64().max(0.001);
                    let eta = (speed > 0.0).then(|| ((total_bytes.saturating_sub(complete_bytes)) as f64 / speed).ceil() as i64);
                    if let Ok(progress) = catalog::open(catalog_path) {
                        let _=progress.execute("UPDATE jobs SET stage='hashing',current_file=?2,stage_processed_items=?3,stage_total_items=?4,stage_processed_bytes=?5,stage_total_bytes=?6,bytes_per_second=?7,estimated_seconds_remaining=?8,updated_at=?9 WHERE id=?1",params![job,path.to_string_lossy(),complete_items as i64,paths.len()as i64,complete_bytes.min(i64::MAX as u64)as i64,total_bytes.min(i64::MAX as u64)as i64,speed,eta,Utc::now().to_rfc3339()]);
                    }
                }
                results.lock().unwrap().insert(cache_key(path), value);
            });
        }
    });
    (results.into_inner().unwrap(), workers, profile.into())
}
fn cache_key(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase()
}

fn metadata_value(v: &serde_json::Value, fallback: &str) -> CapturedMetadata {
    let raw = v
        .get("DateTimeOriginal")
        .or_else(|| v.get("CreateDate"))
        .and_then(|x| x.as_str());
    let date = raw
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok())
        .map(|d| d.and_utc().to_rfc3339())
        .unwrap_or_else(|| fallback.to_string());
    let source = if v.get("DateTimeOriginal").is_some() {
        "exif_original"
    } else if v.get("CreateDate").is_some() {
        "media_created"
    } else {
        "filesystem_modified"
    };
    (
        date,
        source.into(),
        v.get("ImageWidth").and_then(|x| x.as_i64()),
        v.get("ImageHeight").and_then(|x| x.as_i64()),
        v.get("Duration").and_then(|x| x.as_f64()),
        v.get("Model").and_then(|x| x.as_str()).map(str::to_string),
        v.get("GPSLatitude").and_then(|x| x.as_f64()),
        v.get("GPSLongitude").and_then(|x| x.as_f64()),
    )
}

fn capture_metadata_batches<F>(
    paths: &[PathBuf],
    cancel: &crate::process::CancellationToken,
    mut progress: F,
) -> BatchedMetadata
where
    F: FnMut(usize, usize, Option<&Path>),
{
    let mut result = HashMap::new();
    let mut validations = HashMap::new();
    let total_batches = paths.len().div_ceil(200);
    for (batch_index, batch) in paths.chunks(200).enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        progress(
            batch_index,
            total_batches,
            batch.first().map(PathBuf::as_path),
        );
        let mut args = vec![
            "-json".into(),
            "-DateTimeOriginal".into(),
            "-CreateDate".into(),
            "-ImageWidth".into(),
            "-ImageHeight".into(),
            "-Duration#".into(),
            "-Model".into(),
            "-GPSLatitude#".into(),
            "-GPSLongitude#".into(),
            "-validate".into(),
            "-warning".into(),
            "-error".into(),
        ];
        args.extend(batch.iter().map(|p| p.as_os_str().to_os_string()));
        if let Ok(output) = crate::process::run(
            crate::process::ProcessSpec::new("ExifTool", "exiftool")
                .args(args)
                .logical("ExifTool metadata batch"),
            cancel,
        ) {
            if let Ok(values) = serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout) {
                for value in values {
                    if let Some(source) = value.get("SourceFile").and_then(|x| x.as_str()) {
                        let key = cache_key(Path::new(source));
                        result.insert(key.clone(), metadata_value(&value, ""));
                        let extension = Path::new(source)
                            .extension()
                            .and_then(|x| x.to_str())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        if matches!(
                            extension.as_str(),
                            "dng" | "cr2" | "cr3" | "nef" | "arw" | "raf" | "orf" | "rw2"
                        ) {
                            let error = value.get("Error").and_then(|x| x.as_str());
                            let warning = value.get("Warning").and_then(|x| x.as_str());
                            validations.insert(
                                key,
                                crate::media::ValidationResult {
                                    state: if error.is_some() {
                                        crate::media::ValidationState::Corrupted
                                    } else {
                                        crate::media::ValidationState::ValidWithoutPreview
                                    },
                                    tool: "exiftool-batch".into(),
                                    details: error
                                        .or(warning)
                                        .unwrap_or("Estrutura RAW verificada em lote")
                                        .to_string(),
                                },
                            );
                        }
                    }
                }
            }
        }
        progress(
            batch_index + 1,
            total_batches,
            batch.last().map(PathBuf::as_path),
        );
    }
    (result, validations)
}

fn technical_photo_batches(
    paths: &[PathBuf],
    cancel: &crate::process::CancellationToken,
) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    for batch in paths.chunks(200) {
        if cancel.is_cancelled() {
            break;
        }
        let mut args = vec![
            "-json".into(),
            "-n".into(),
            "-LensModel".into(),
            "-ISO".into(),
            "-FNumber".into(),
            "-ExposureTime".into(),
            "-FocalLength".into(),
            "-Orientation#".into(),
            "-ColorSpace".into(),
            "-PreviewImageLength".into(),
        ];
        args.extend(batch.iter().map(|path| path.as_os_str().to_os_string()));
        let Ok(output) = crate::process::run(
            crate::process::ProcessSpec::new("ExifTool", "exiftool")
                .args(args)
                .timeout(std::time::Duration::from_secs(60))
                .logical("ExifTool technical inventory batch"),
            cancel,
        ) else {
            continue;
        };
        if let Ok(values) = serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout) {
            for value in values {
                if let Some(source) = value.get("SourceFile").and_then(|item| item.as_str()) {
                    result.insert(cache_key(Path::new(source)), value);
                }
            }
        }
    }
    result
}

fn ignored(entry: &DirEntry) -> bool {
    crate::pipeline::ignored(entry)
}
#[cfg(test)]
fn hash_file(path: &Path) -> Result<String, String> {
    crate::storage::sha256(path)
}
fn media_type(ext: &str) -> &'static str {
    crate::pipeline::media_type(ext)
}
fn system_time(meta: &fs::Metadata) -> String {
    meta.modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(Utc::now)
        .to_rfc3339()
}
type CaptureMetadata = (
    String,
    String,
    Option<i64>,
    Option<i64>,
    Option<f64>,
    Option<String>,
    Option<f64>,
    Option<f64>,
);
fn capture_metadata(
    path: &Path,
    fallback: &str,
    cancel: &crate::process::CancellationToken,
) -> CaptureMetadata {
    let output = crate::process::run(
        crate::process::ProcessSpec::new("ExifTool", "exiftool")
            .args([
                std::ffi::OsStr::new("-json"),
                std::ffi::OsStr::new("-DateTimeOriginal"),
                std::ffi::OsStr::new("-CreateDate"),
                std::ffi::OsStr::new("-ImageWidth"),
                std::ffi::OsStr::new("-ImageHeight"),
                std::ffi::OsStr::new("-Duration#"),
                std::ffi::OsStr::new("-Model"),
                std::ffi::OsStr::new("-GPSLatitude#"),
                std::ffi::OsStr::new("-GPSLongitude#"),
                path.as_os_str(),
            ])
            .logical("ExifTool metadata"),
        cancel,
    );
    if let Ok(out) = output {
        if let Ok(values) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
            if let Some(v) = values.first() {
                return metadata_value(v, fallback);
            }
        }
    }
    (
        fallback.into(),
        "filesystem_modified".into(),
        None,
        None,
        None,
        None,
        None,
        None,
    )
}
#[cfg(test)]
fn copy_verified(source: &Path, destination: &Path, expected: &str) -> Result<(), String> {
    crate::storage::copy_verified(source, destination, expected)
}
fn event(conn: &rusqlite::Connection, job: &str, path: &str, state: &str, details: &str) {
    let _ = conn.execute(
        "INSERT INTO events(job_id,at,path,state,details)VALUES(?1,?2,?3,?4,?5)",
        params![job, Utc::now().to_rfc3339(), path, state, details],
    );
}

pub fn job_progress(cfg: &LibraryConfig, job: &str) -> Result<JobProgress, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    conn.query_row("SELECT id,state,stage,current_file,processed_items,total_items,processed_bytes,total_bytes,imported_count,duplicate_count,excluded_count,failed_count,stage_processed_items,stage_total_items,stage_processed_bytes,stage_total_bytes,bytes_per_second,estimated_seconds_remaining,library_state,backup_state FROM jobs WHERE id=?1",[job],|r|{let state:String=r.get(1)?;let pi:i64=r.get(4)?;let ti:i64=r.get(5)?;let pb:i64=r.get(6)?;let tb:i64=r.get(7)?;let spi:i64=r.get(12)?;let sti:i64=r.get(13)?;let spb:i64=r.get(14)?;let stb:i64=r.get(15)?;let stage=if stb>0{spb as f64/stb as f64*100.0}else if sti>0{spi as f64/sti as f64*100.0}else{0.0};let overall=if state=="analyzing"{stage}else if tb>0{pb as f64/tb as f64*100.0}else if ti>0{pi as f64/ti as f64*100.0}else{0.0};Ok(JobProgress{job_id:r.get(0)?,state,stage:r.get(2)?,current_file:r.get(3)?,processed_items:pi,total_items:ti,processed_bytes:pb,total_bytes:tb,imported:r.get(8)?,duplicates:r.get(9)?,excluded:r.get(10)?,failed:r.get(11)?,stage_percent:stage.clamp(0.0,100.0),overall_percent:overall.clamp(0.0,100.0),bytes_per_second:r.get(16)?,estimated_seconds_remaining:r.get(17)?,library_state:r.get(18)?,backup_state:r.get(19)?})}).map_err(|e|e.to_string())
}

pub fn import_summary(cfg: &LibraryConfig, job: &str) -> Result<ImportSummary, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let (source_id, source_path): (String, String) = conn
        .query_row(
            "SELECT source_id,source_path FROM jobs WHERE id=?1",
            [job],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    let (discovered,new_files,duplicates,invalid,required,excluded):(i64,i64,i64,i64,i64,i64)=conn.query_row("SELECT COUNT(*),COALESCE(SUM(state='new'),0),COALESCE(SUM(state='duplicate'),0),COALESCE(SUM(state='review'),0),COALESCE(SUM(CASE WHEN state='new' THEN bytes ELSE 0 END),0),(SELECT excluded_count FROM jobs WHERE id=?1) FROM job_items WHERE job_id=?1",[job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).map_err(|e|e.to_string())?;
    let issues = import_issues(&conn, job)?;
    Ok(ImportSummary {
        job_id: job.into(),
        source_id,
        source_path,
        discovered,
        new_files,
        duplicates,
        invalid,
        required_bytes: required,
        available_bytes: fs2::available_space(Path::new(&cfg.master_path)).unwrap_or(0),
        excluded,
        issues,
    })
}

fn import_issues(
    conn: &rusqlite::Connection,
    job: &str,
) -> Result<Vec<crate::models::ImportIssue>, String> {
    let mut statement = conn.prepare("SELECT COALESCE(validation_state,'unreadable'),extension,COUNT(*),COALESCE(SUM(bytes),0) FROM job_items WHERE job_id=?1 AND state='review' GROUP BY validation_state,extension ORDER BY COUNT(*) DESC").map_err(|e| e.to_string())?;
    let issues = statement
        .query_map([job], |row| {
            let kind: String = row.get(0)?;
            let message = match kind.as_str() {
                "missing_dependency" => {
                    "O componente necessário para verificar estes arquivos não iniciou."
                }
                "timeout" => "A verificação excedeu o tempo limite.",
                "unsupported_format" => "Este formato ainda não pode ser validado.",
                "corrupted" => "O conteúdo não pôde ser decodificado e precisa de revisão.",
                _ => "O arquivo não pôde ser lido e precisa de revisão.",
            }
            .to_string();
            Ok(crate::models::ImportIssue {
                kind,
                extension: row.get(1)?,
                items: row.get(2)?,
                bytes: row.get(3)?,
                message,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(issues)
}

fn control_point(conn: &rusqlite::Connection, job: &str) -> Result<(), String> {
    loop {
        let state: String = conn
            .query_row("SELECT state FROM jobs WHERE id=?1", [job], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        match state.as_str() {
            "pausing" | "paused" => {
                conn.execute(
                    "UPDATE jobs SET state='paused',updated_at=?2 WHERE id=?1",
                    params![job, Utc::now().to_rfc3339()],
                )
                .ok();
                std::thread::sleep(std::time::Duration::from_millis(150));
            }
            "canceling" | "canceled" => {
                conn.execute(
                    "UPDATE jobs SET state='canceled',finished_at=?2,updated_at=?2 WHERE id=?1",
                    params![job, Utc::now().to_rfc3339()],
                )
                .ok();
                return Err("JOB_CANCELED".into());
            }
            _ => return Ok(()),
        }
    }
}

pub fn set_job_state(cfg: &LibraryConfig, job: &str, state: &str) -> Result<JobProgress, String> {
    let mut conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let (stage, current_state): (String, String) = conn
        .query_row("SELECT stage,state FROM jobs WHERE id=?1", [job], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|e| e.to_string())?;
    let normalized = match state {
        "paused" => "pausing",
        "running" => {
            if matches!(stage.as_str(), "backup" | "backup_space_check") {
                "protecting"
            } else if matches!(
                stage.as_str(),
                "discovery" | "validation" | "metadata" | "hashing" | "deduplication"
            ) {
                "analyzing"
            } else {
                "consolidating"
            }
        }
        "canceled" => "canceling",
        _ => return Err("Estado de controle inválido".into()),
    };
    if !crate::pipeline::valid_transition(&current_state, normalized) {
        return Err(format!(
            "Transição de estado inválida: {current_state} → {normalized}"
        ));
    }
    let transaction = conn.transaction().map_err(|e| e.to_string())?;
    transaction
        .execute(
            "UPDATE jobs SET state=?2,updated_at=?3 WHERE id=?1",
            params![job, normalized, Utc::now().to_rfc3339()],
        )
        .map_err(|e| e.to_string())?;
    let details = match normalized {
        "pausing" => "Pausa solicitada",
        "canceling" => "Cancelamento solicitado",
        _ => "Processamento retomado",
    };
    event(&transaction, job, "", normalized, details);
    transaction.commit().map_err(|e| e.to_string())?;
    job_progress(cfg, job)
}

pub fn queue_analysis(
    cfg: &LibraryConfig,
    source_path: &str,
    source_name: &str,
) -> Result<String, String> {
    let root = Path::new(source_path);
    if !root.is_dir() {
        return Err(format!("A fonte não está acessível: {source_path}"));
    }
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let identity = crate::volume::identify(root)?;
    let source_key = crate::volume::source_key(&identity, root);
    let source_id = conn
        .query_row("SELECT id FROM sources WHERE path=?1 OR (mount_path=?2 AND volume_id=volume_label) ORDER BY path=?1 DESC LIMIT 1", params![&source_key,source_path], |r| {
            r.get::<_, String>(0)
        })
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    conn.execute("INSERT INTO sources(id,name,path,volume_label,volume_id,mount_path,available)VALUES(?1,?2,?3,?4,?5,?6,1)ON CONFLICT(id)DO UPDATE SET name=excluded.name,path=excluded.path,volume_label=excluded.volume_label,volume_id=excluded.volume_id,mount_path=excluded.mount_path,available=1",params![source_id,source_name,source_key,identity.label,identity.id,source_path]).map_err(|e|e.to_string())?;
    let job = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at)VALUES(?1,?2,?3,'queued','discovery',?4,?4)",params![job,source_id,source_path,now]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO job_counters(job_id) VALUES(?1)", [&job])
        .map_err(|e| e.to_string())?;
    Ok(job)
}

pub fn analyze(
    cfg: &LibraryConfig,
    source_path: &str,
    source_name: &str,
) -> Result<ImportSummary, String> {
    analyze_with_job_cancel(
        cfg,
        source_path,
        source_name,
        None,
        &crate::process::CancellationToken::default(),
    )
}
#[cfg(test)]
pub fn analyze_with_job(
    cfg: &LibraryConfig,
    source_path: &str,
    source_name: &str,
    requested_job: Option<&str>,
) -> Result<ImportSummary, String> {
    analyze_with_job_cancel(
        cfg,
        source_path,
        source_name,
        requested_job,
        &crate::process::CancellationToken::default(),
    )
}
pub fn analyze_with_job_cancel(
    cfg: &LibraryConfig,
    source_path: &str,
    source_name: &str,
    requested_job: Option<&str>,
    cancel: &crate::process::CancellationToken,
) -> Result<ImportSummary, String> {
    let analysis_started = Instant::now();
    let root = Path::new(source_path);
    if !root.is_dir() {
        return Err(format!("A fonte não está acessível: {source_path}"));
    }
    let source_abs = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let master_abs = fs::canonicalize(&cfg.master_path).map_err(|e| e.to_string())?;
    let backup_abs = fs::canonicalize(&cfg.backup_path).map_err(|e| e.to_string())?;
    if source_abs.starts_with(&master_abs)
        || master_abs.starts_with(&source_abs)
        || source_abs.starts_with(&backup_abs)
        || backup_abs.starts_with(&source_abs)
    {
        return Err(
            "A fonte não pode conter nem estar contida nas pastas do acervo ou backup".into(),
        );
    }
    let catalog_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let mut conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
    let identity = crate::volume::identify(root)?;
    let source_key = crate::volume::source_key(&identity, root);
    let source_id = conn
        .query_row("SELECT id FROM sources WHERE path=?1 OR (mount_path=?2 AND volume_id=volume_label) ORDER BY path=?1 DESC LIMIT 1", params![&source_key,source_path], |r| {
            r.get::<_, String>(0)
        })
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    conn.execute("INSERT INTO sources(id,name,path,volume_label,volume_id,mount_path,available,last_scan)VALUES(?1,?2,?3,?4,?5,?6,1,?7) ON CONFLICT(id) DO UPDATE SET name=excluded.name,path=excluded.path,volume_label=excluded.volume_label,volume_id=excluded.volume_id,mount_path=excluded.mount_path,available=1,last_scan=excluded.last_scan",params![source_id,source_name,source_key,identity.label,identity.id,source_path,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    let job = requested_job
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at,stage)VALUES(?1,?2,?3,'analyzing',?4,?4,'discovery')ON CONFLICT(id)DO UPDATE SET source_id=excluded.source_id,state='analyzing',stage='discovery',updated_at=excluded.updated_at",params![job,source_id,source_path,now]).map_err(|e|e.to_string())?;
    let mut discovered = 0;
    let mut new_files = 0;
    let mut duplicates = 0;
    let mut invalid = 0;
    let mut excluded = 0;
    let mut required = 0;
    let mut session_hashes = HashSet::new();
    let mut scan_total = 0i64;
    let mut scan_bytes = 0i64;
    let mut validation_ms = 0i64;
    let mut media_paths = Vec::new();
    let mut size_counts: HashMap<u64, usize> = HashMap::new();
    let inventory_started = Instant::now();
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !ignored(e))
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            let ext = entry
                .path()
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if MEDIA.contains(&ext.as_str()) {
                scan_total += 1;
                scan_bytes += entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
                if let Ok(metadata) = entry.metadata() {
                    *size_counts.entry(metadata.len()).or_default() += 1;
                }
                media_paths.push(entry.path().to_path_buf());
            } else {
                excluded += 1;
            }
        }
    }
    {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for path in &media_paths {
            if let Ok(meta) = fs::metadata(path) {
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                tx.execute("INSERT INTO job_items(job_id,source_path,filename,extension,media_type,bytes,modified_at,current_stage,state,created_at,updated_at)VALUES(?1,?2,?3,?4,?5,?6,?7,'inventory','inventoried',?8,?8)ON CONFLICT(job_id,source_path)DO UPDATE SET bytes=excluded.bytes,modified_at=excluded.modified_at,current_stage='inventory',updated_at=excluded.updated_at",params![job,path.to_string_lossy(),path.file_name().unwrap_or_default().to_string_lossy(),ext,media_type(&ext),meta.len()as i64,system_time(&meta),Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
    }
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'inventory',?2,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,inventory_started.elapsed().as_millis()as i64,scan_total,scan_bytes,Utc::now().to_rfc3339()]).ok();
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'inventory_walks',0,1,?2,?3)ON CONFLICT(job_id,stage)DO UPDATE SET items=1,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,scan_bytes,Utc::now().to_rfc3339()]).ok();
    event(
        &conn,
        &job,
        source_path,
        "inventory_ready",
        &format!("Inventário rápido concluído: {scan_total} mídias, {scan_bytes} bytes"),
    );
    conn.execute("UPDATE jobs SET stage='confirmation',total_items=?2,total_bytes=?3,stage_total_items=?2,stage_total_bytes=?3,processed_items=0,processed_bytes=0,stage_processed_items=0,stage_processed_bytes=0 WHERE id=?1",params![job,scan_total,scan_bytes]).map_err(|e|e.to_string())?;
    let mut uncached = Vec::new();
    let mut uncached_bytes = 0u64;
    let mut hash_candidates = HashSet::new();
    let mut cache_hits = 0i64;
    let mut deferred_hash_items = 0i64;
    for path in &media_paths {
        if let Ok(meta) = fs::metadata(path) {
            let modified = system_time(&meta);
            let cached:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM job_items WHERE job_id<>?1 AND source_path=?2 AND bytes=?3 AND modified_at=?4 AND sha256 IS NOT NULL AND state IN('new','duplicate','consolidated'))",params![job,path.to_string_lossy(),meta.len()as i64,modified],|r|r.get(0)).unwrap_or(false);
            if cached {
                cache_hits += 1;
            }
            let catalog_same_size: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM assets WHERE bytes=?1)",
                    [meta.len().min(i64::MAX as u64) as i64],
                    |r| r.get(0),
                )
                .unwrap_or(false);
            let needs_hash =
                size_counts.get(&meta.len()).copied().unwrap_or(0) > 1 || catalog_same_size;
            if !cached && needs_hash {
                uncached_bytes = uncached_bytes.saturating_add(meta.len());
                uncached.push(path.clone());
                hash_candidates.insert(cache_key(path));
            } else if !cached {
                deferred_hash_items += 1;
            }
        }
    }
    drop(conn);
    let hashing_started = Instant::now();
    let (mut hash_cache, hash_workers, storage_profile) =
        hash_files_adaptive(uncached, &catalog_path, &job, uncached_bytes, cancel);
    let hashing_ms = hashing_started.elapsed().as_millis() as i64;
    conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'hashing_workers',?2,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,hashing_ms,hash_workers as i64,uncached_bytes.min(i64::MAX as u64)as i64,Utc::now().to_rfc3339()]).ok();
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,?2,0,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,format!("storage_profile:{storage_profile}"),hash_workers as i64,uncached_bytes.min(i64::MAX as u64)as i64,Utc::now().to_rfc3339()]).ok();
    for (stage, items, bytes) in [
        (
            "deferred_hash",
            deferred_hash_items,
            scan_bytes - uncached_bytes.min(i64::MAX as u64) as i64,
        ),
        ("cache_hits", cache_hits, 0),
    ] {
        conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,?2,0,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,stage,items,bytes,Utc::now().to_rfc3339()]).ok();
    }
    conn.execute("UPDATE jobs SET stage='metadata',current_file='Preparando metadados em lotes',updated_at=?2 WHERE id=?1",params![job,Utc::now().to_rfc3339()]).ok();
    drop(conn);
    let metadata_started = Instant::now();
    let progress_catalog = catalog_path.clone();
    let progress_job = job.clone();
    let (mut metadata_cache, mut validation_cache) = capture_metadata_batches(
        &media_paths,
        cancel,
        |done, total, current| {
            if let Ok(progress_conn) = catalog::open(&progress_catalog) {
                let current = current
                    .map(|path| {
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_default();
                let message = if total > 0 {
                    format!("Lote {done} de {total} · {current}")
                } else {
                    "Preparando informações das mídias".to_string()
                };
                let _ = progress_conn.execute(
                "UPDATE jobs SET stage='metadata',current_file=?2,stage_processed_items=?3,stage_total_items=?4,updated_at=?5 WHERE id=?1",
                params![progress_job,message,done as i64,total as i64,Utc::now().to_rfc3339()],
            );
            }
        },
    );
    conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'metadata_batch',?2,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,metadata_started.elapsed().as_millis() as i64,scan_total,scan_bytes,Utc::now().to_rfc3339()]).ok();
    conn.execute(
        "UPDATE jobs SET stage='validation',current_file=NULL,stage_processed_items=0,stage_total_items=?2,stage_processed_bytes=0,stage_total_bytes=?3,bytes_per_second=NULL,estimated_seconds_remaining=NULL,updated_at=?4 WHERE id=?1",
        params![job,scan_total,scan_bytes,Utc::now().to_rfc3339()],
    )
    .ok();
    if cancel.is_cancelled() {
        let now = Utc::now().to_rfc3339();
        conn.execute("UPDATE jobs SET state='canceled',interruption_reason='Cancelado pelo usuário',finished_at=?2,updated_at=?2,current_file=NULL WHERE id=?1",params![job,now]).ok();
        event(
            &conn,
            &job,
            source_path,
            "canceled",
            "Análise cancelada pelo usuário",
        );
        return Err("JOB_CANCELED".into());
    }
    let mut scan_processed = 0i64;
    let mut scan_processed_bytes = 0i64;
    for inventoried_path in &media_paths {
        let path = inventoried_path.as_path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !MEDIA.contains(&ext.as_str()) {
            excluded += 1;
            event(
                &conn,
                &job,
                &path.to_string_lossy(),
                "excluded",
                "Formato ignorado pela política de importação",
            );
            continue;
        }
        discovered += 1;
        control_point(&conn, &job)?;
        if progress_due(scan_processed, scan_total) {
            conn.execute("UPDATE jobs SET current_file=?2,stage='validation',stage_processed_items=?3,stage_processed_bytes=?4,updated_at=?5 WHERE id=?1",params![job,path.to_string_lossy(),scan_processed,scan_processed_bytes,Utc::now().to_rfc3339()]).ok();
        }
        match fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len() as i64;
                let modified = system_time(&meta);
                let prior:Option<(Option<String>,String,i64)>=conn.query_row("SELECT ji.sha256,CASE WHEN EXISTS(SELECT 1 FROM assets a WHERE a.hash=ji.sha256) THEN 'duplicate' ELSE ji.state END,ji.id FROM job_items ji WHERE ji.source_path=?1 AND ji.bytes=?2 AND ji.modified_at=?3 AND ji.sha256 IS NOT NULL AND ji.state IN('new','duplicate','consolidated') ORDER BY (ji.job_id=?4) DESC,ji.id DESC LIMIT 1",params![path.to_string_lossy(),size,modified,job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|e|e.to_string())?;
                if let Some((Some(hash), status, cached_item_id)) = prior {
                    if matches!(status.as_str(), "new" | "duplicate") {
                        if status == "duplicate" {
                            duplicates += 1;
                            event(
                                &conn,
                                &job,
                                &path.to_string_lossy(),
                                "duplicate",
                                "Conteúdo SHA-256 já inventariado",
                            );
                        } else {
                            new_files += 1;
                            required += size
                        }
                        session_hashes.insert(hash.clone());
                        conn.execute("INSERT INTO job_items(job_id,source_path,filename,extension,media_type,bytes,modified_at,sha256,current_stage,state,validation_state,created_at,updated_at,captured_at,date_source,width,height,duration,camera,latitude,longitude)SELECT ?1,?2,filename,extension,media_type,bytes,modified_at,sha256,'deduplication',?3,validation_state,?4,?4,captured_at,date_source,width,height,duration,camera,latitude,longitude FROM job_items WHERE id=?5 ON CONFLICT(job_id,source_path)DO UPDATE SET sha256=excluded.sha256,state=excluded.state,current_stage='deduplication',updated_at=excluded.updated_at",params![job,path.to_string_lossy(),status,Utc::now().to_rfc3339(),cached_item_id]).map_err(|e|e.to_string())?;
                        conn.execute("INSERT OR IGNORE INTO pending_files(job_id,path,filename,extension,media_type,bytes,modified_at,hash,status)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![job,path.to_string_lossy(),path.file_name().unwrap_or_default().to_string_lossy(),ext,media_type(&ext),size,system_time(&meta),hash,status]).map_err(|e|e.to_string())?;
                        event(
                            &conn,
                            &job,
                            &path.to_string_lossy(),
                            "cache_hit",
                            "Validação, metadados e hash reutilizados de uma análise anterior",
                        );
                        scan_processed += 1;
                        scan_processed_bytes += size;
                        if progress_due(scan_processed, scan_total) {
                            conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,stage_processed_items=?2,stage_processed_bytes=?3 WHERE id=?1",params![job,scan_processed,scan_processed_bytes]).ok();
                        }
                        continue;
                    }
                }
                drop(conn);
                let validation_started = Instant::now();
                let checked = validation_cache
                    .remove(&cache_key(path))
                    .unwrap_or_else(|| crate::media::validate(path, &ext, cancel));
                let validation_duration = validation_started.elapsed().as_millis() as i64;
                validation_ms += validation_duration;
                conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
                if cancel.is_cancelled() {
                    conn.execute(
                        "UPDATE jobs SET state='canceled',finished_at=?2,updated_at=?2 WHERE id=?1",
                        params![job, Utc::now().to_rfc3339()],
                    )
                    .ok();
                    return Err("JOB_CANCELED".into());
                }
                let item_id=conn.query_row("INSERT INTO job_items(job_id,source_path,filename,extension,media_type,bytes,modified_at,current_stage,state,validation_state,created_at,updated_at)VALUES(?1,?2,?3,?4,?5,?6,?7,'validation',?8,?9,?10,?10)ON CONFLICT(job_id,source_path)DO UPDATE SET state=excluded.state,validation_state=excluded.validation_state,updated_at=excluded.updated_at RETURNING id",params![job,path.to_string_lossy(),path.file_name().unwrap_or_default().to_string_lossy(),ext,media_type(&ext),size,system_time(&meta),if checked.state.accepted(){"processing"}else{"review"},checked.state.as_str(),Utc::now().to_rfc3339()],|r|r.get::<_,i64>(0)).map_err(|e|e.to_string())?;
                conn.execute("INSERT INTO media_validation(job_item_id,state,tool,details,checked_at)VALUES(?1,?2,?3,?4,?5)ON CONFLICT(job_item_id)DO UPDATE SET state=excluded.state,tool=excluded.tool,details=excluded.details,checked_at=excluded.checked_at",params![item_id,checked.state.as_str(),checked.tool,checked.details,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                conn.execute("INSERT INTO process_events(job_id,job_item_id,at,tool,logical_command,duration_ms,exit_code,state,error_kind,details)VALUES(?1,?2,?3,?4,'media validation',?5,?6,?7,?8,?9)",params![job,item_id,Utc::now().to_rfc3339(),checked.tool,validation_duration,if checked.state.accepted(){Some(0)}else{None},if checked.state.accepted(){"completed"}else{"failed"},if checked.state.accepted(){None::<String>}else{Some(checked.state.as_str().to_string())},crate::process::sanitize(&checked.details)]).ok();
                if !checked.state.accepted() {
                    invalid += 1;
                    event(
                        &conn,
                        &job,
                        &path.to_string_lossy(),
                        "validation_failed",
                        checked.state.as_str(),
                    );
                    scan_processed += 1;
                    scan_processed_bytes += size;
                    if progress_due(scan_processed, scan_total) {
                        conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,stage_processed_items=?2,stage_processed_bytes=?3 WHERE id=?1",params![job,scan_processed,scan_processed_bytes]).ok();
                    }
                    continue;
                }
                conn.execute(
                    "UPDATE jobs SET stage='metadata',current_file=?2,updated_at=?3 WHERE id=?1",
                    params![job, path.to_string_lossy(), Utc::now().to_rfc3339()],
                )
                .ok();
                drop(conn);
                let fallback_time = system_time(&meta);
                let (captured, date_source, width, height, duration, camera, latitude, longitude) =
                    metadata_cache
                        .remove(&cache_key(path))
                        .map(|mut value| {
                            if value.0.is_empty() {
                                value.0 = fallback_time.clone()
                            }
                            value
                        })
                        .unwrap_or_else(|| capture_metadata(path, &fallback_time, cancel));
                conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
                if cancel.is_cancelled() {
                    return Err("JOB_CANCELED".into());
                }
                conn.execute("UPDATE job_items SET captured_at=?2,date_source=?3,width=?4,height=?5,duration=?6,camera=?7,latitude=?8,longitude=?9,current_stage='metadata',updated_at=?10 WHERE id=?1",params![item_id,captured,date_source,width,height,duration,camera,latitude,longitude,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                scan_processed += 1;
                scan_processed_bytes += size;
                if progress_due(scan_processed, scan_total) {
                    conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,stage_processed_items=?2,stage_processed_bytes=?3 WHERE id=?1",params![job,scan_processed,scan_processed_bytes]).ok();
                }
                let key = cache_key(path);
                if !hash_candidates.contains(&key) && !hash_cache.contains_key(&key) {
                    new_files += 1;
                    required += size;
                    conn.execute("UPDATE job_items SET sha256=NULL,current_stage='deduplication',state='new',updated_at=?2 WHERE id=?1",params![item_id,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                    conn.execute("INSERT INTO pending_files(job_id,path,filename,extension,media_type,bytes,modified_at,hash,status)VALUES(?1,?2,?3,?4,?5,?6,?7,NULL,'new')ON CONFLICT(job_id,path)DO UPDATE SET filename=excluded.filename,extension=excluded.extension,media_type=excluded.media_type,bytes=excluded.bytes,modified_at=excluded.modified_at,status='new',error=NULL",params![job,path.to_string_lossy(),path.file_name().unwrap_or_default().to_string_lossy(),ext,media_type(&ext),size,system_time(&meta)]).map_err(|e|e.to_string())?;
                    event(
                        &conn,
                        &job,
                        &path.to_string_lossy(),
                        "hash_deferred",
                        "Sem candidato do mesmo tamanho; SHA-256 será calculado durante a cópia",
                    );
                    continue;
                }
                conn.execute(
                    "UPDATE jobs SET stage='hashing',current_file=?2,updated_at=?3 WHERE id=?1",
                    params![job, path.to_string_lossy(), Utc::now().to_rfc3339()],
                )
                .ok();
                drop(conn);
                let hash_result = hash_cache
                    .remove(&key)
                    .unwrap_or_else(|| crate::storage::sha256_cancel(path, Some(cancel)));
                conn = catalog::open(&catalog_path).map_err(|e| e.to_string())?;
                match hash_result {
                    Ok(hash) => {
                        let in_catalog: bool = conn
                            .query_row(
                                "SELECT EXISTS(SELECT 1 FROM assets WHERE hash=?1)",
                                [&hash],
                                |r| r.get(0),
                            )
                            .unwrap_or(false);
                        let duplicate = in_catalog || !session_hashes.insert(hash.clone());
                        conn.execute("UPDATE jobs SET stage='deduplication',current_file=?2,updated_at=?3 WHERE id=?1",params![job,path.to_string_lossy(),Utc::now().to_rfc3339()]).ok();
                        let status = if duplicate {
                            duplicates += 1;
                            event(
                                &conn,
                                &job,
                                &path.to_string_lossy(),
                                "duplicate",
                                "Conteúdo SHA-256 já inventariado",
                            );
                            "duplicate"
                        } else {
                            new_files += 1;
                            required += size;
                            "new"
                        };
                        conn.execute("UPDATE job_items SET sha256=?2,current_stage='deduplication',state=?3,updated_at=?4 WHERE id=?1",params![item_id,hash,status,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                        conn.execute("INSERT INTO pending_files(job_id,path,filename,extension,media_type,bytes,modified_at,hash,status)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![job,path.to_string_lossy(),path.file_name().unwrap_or_default().to_string_lossy(),ext,media_type(&ext),size,system_time(&meta),hash,status]).map_err(|e|e.to_string())?;
                    }
                    Err(err) => {
                        if err == "JOB_CANCELED" {
                            return Err(err);
                        }
                        invalid += 1;
                        conn.execute("UPDATE job_items SET state='review',last_error_kind='unreadable',last_error=?2 WHERE id=?1",params![item_id,err]).ok();
                        event(
                            &conn,
                            &job,
                            &path.to_string_lossy(),
                            "failed",
                            "Falha no SHA-256",
                        )
                    }
                }
            }
            Err(err) => {
                invalid += 1;
                event(
                    &conn,
                    &job,
                    &path.to_string_lossy(),
                    "failed",
                    &err.to_string(),
                )
            }
        }
    }
    conn.execute("INSERT OR IGNORE INTO job_selection(job_id,job_item_id,selected,batch_no) SELECT ?1,id,1,1 FROM job_items WHERE job_id=?1 AND state='new'",[&job]).map_err(|e|e.to_string())?;
    conn.execute("UPDATE jobs SET state='ready',stage='ready',processed_items=?2,total_items=?2,processed_bytes=?3,total_bytes=?3,imported_count=0,duplicate_count=?4,excluded_count=?5,failed_count=?6,stage_processed_items=?2,stage_total_items=?2,stage_processed_bytes=?3,stage_total_bytes=?3,updated_at=?7 WHERE id=?1",params![job,discovered,scan_bytes,duplicates,excluded,invalid,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'analysis_total',?2,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,analysis_started.elapsed().as_millis() as i64,discovered,scan_bytes,Utc::now().to_rfc3339()]).ok();
    for (stage, duration) in [
        ("validation_total", validation_ms),
        ("hashing_total", hashing_ms),
    ] {
        conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,?2,?3,?4,?5,?6)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,stage,duration,discovered,scan_bytes,Utc::now().to_rfc3339()]).ok();
    }
    conn.execute("UPDATE job_counters SET imported=0,duplicates=?2,excluded=?3,failed=?4,validated=?5 WHERE job_id=?1",params![job,duplicates,excluded,invalid,discovered-invalid]).map_err(|e|e.to_string())?;
    let known_paths = {
        let mut statement = conn
            .prepare("SELECT path FROM occurrences WHERE source_id=?1")
            .map_err(|e| e.to_string())?;
        let values = statement
            .query_map([&source_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    for known in known_paths {
        if !Path::new(&known).exists() {
            event(
                &conn,
                &job,
                &known,
                "source_file_missing",
                "Arquivo não encontrado em uma fonte conectada",
            );
        }
    }
    event(
        &conn,
        &job,
        source_path,
        "completed",
        &format!("Análise concluída: {discovered} mídias encontradas"),
    );
    let available = fs2::available_space(Path::new(&cfg.master_path)).unwrap_or(0);
    let issues = import_issues(&conn, &job)?;
    Ok(ImportSummary {
        job_id: job,
        source_id,
        source_path: source_path.into(),
        discovered,
        new_files,
        duplicates,
        invalid,
        required_bytes: required,
        available_bytes: available,
        excluded,
        issues,
    })
}

pub fn storage_plan(cfg: &LibraryConfig, job: &str) -> Result<crate::models::StoragePlan, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let required: i64 = conn.query_row(
        "SELECT COALESCE(SUM(p.bytes),0) FROM pending_files p JOIN job_items ji ON ji.job_id=p.job_id AND ji.source_path=p.path LEFT JOIN job_selection s ON s.job_id=ji.job_id AND s.job_item_id=ji.id WHERE p.job_id=?1 AND p.status IN('new','failed_copy') AND COALESCE(s.selected,1)=1",
        [job],
        |r| r.get(0),
    ).map_err(|e| e.to_string())?;
    let required = required.max(0) as u64;
    let reserve = (required / 50).max(1024 * 1024 * 1024);
    let master = Path::new(&cfg.master_path);
    let backup = Path::new(&cfg.backup_path);
    let master_available = fs2::available_space(master)
        .map_err(|e| format!("Não foi possível verificar o espaço da biblioteca: {e}"))?;
    let backup_available = fs2::available_space(backup)
        .map_err(|e| format!("Não foi possível verificar o espaço da réplica: {e}"))?;
    let root = |path: &Path| {
        path.components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_ascii_lowercase())
    };
    let same_volume = root(master) == root(backup);
    let master_needed = required.saturating_add(reserve);
    let master_missing = master_needed.saturating_sub(master_available);
    let backup_needed = if same_volume {
        required.saturating_mul(2).saturating_add(reserve)
    } else {
        required.saturating_add(reserve)
    };
    let backup_missing = backup_needed.saturating_sub(backup_available);
    let safe_capacity = master_available.saturating_sub(reserve);
    let (selected_items, selected_bytes): (i64, i64) = conn.query_row("SELECT COUNT(*),COALESCE(SUM(ji.bytes),0) FROM job_items ji LEFT JOIN job_selection s ON s.job_id=ji.job_id AND s.job_item_id=ji.id WHERE ji.job_id=?1 AND ji.state='new' AND COALESCE(s.selected,1)=1",[job],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|e|e.to_string())?;
    let maximum_safe_items: i64 = conn.query_row("SELECT COUNT(*) FROM (SELECT ji.bytes,SUM(ji.bytes) OVER(ORDER BY ji.bytes,ji.id) running FROM job_items ji WHERE ji.job_id=?1 AND ji.state='new') WHERE running<=?2",params![job,safe_capacity.min(i64::MAX as u64) as i64],|r|r.get(0)).map_err(|e|e.to_string())?;
    Ok(crate::models::StoragePlan {
        master_required_bytes: required,
        backup_required_bytes: required,
        reserve_bytes: reserve,
        master_available_bytes: master_available,
        backup_available_bytes: backup_available,
        same_volume,
        can_consolidate: master_missing == 0,
        can_protect: backup_missing == 0,
        missing_bytes: master_missing,
        backup_missing_bytes: backup_missing,
        selected_items,
        selected_bytes: selected_bytes.max(0) as u64,
        maximum_safe_bytes: safe_capacity,
        maximum_safe_items,
    })
}

pub fn apply_selection(
    cfg: &LibraryConfig,
    request: &crate::models::SelectionRequest,
) -> Result<crate::models::SelectionResult, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    conn.execute("INSERT OR IGNORE INTO job_selection(job_id,job_item_id,selected,batch_no) SELECT ?1,id,1,1 FROM job_items WHERE job_id=?1 AND state='new'",[&request.job_id]).map_err(|e|e.to_string())?;
    conn.execute(
        "UPDATE job_selection SET selected=0 WHERE job_id=?1",
        [&request.job_id],
    )
    .map_err(|e| e.to_string())?;
    match request.mode.as_str() {
        "all" => {
            conn.execute(
                "UPDATE job_selection SET selected=1 WHERE job_id=?1",
                [&request.job_id],
            )
            .map_err(|e| e.to_string())?;
        }
        "media_type" => {
            conn.execute("UPDATE job_selection SET selected=1 WHERE job_id=?1 AND job_item_id IN(SELECT id FROM job_items WHERE job_id=?1 AND media_type=?2 AND state='new')",params![request.job_id,request.value.as_deref().unwrap_or("")]).map_err(|e|e.to_string())?;
        }
        "year" => {
            conn.execute("UPDATE job_selection SET selected=1 WHERE job_id=?1 AND job_item_id IN(SELECT id FROM job_items WHERE job_id=?1 AND substr(captured_at,1,4)=?2 AND state='new')",params![request.job_id,request.value.as_deref().unwrap_or("")]).map_err(|e|e.to_string())?;
        }
        "folder" => {
            let prefix = format!("{}%", request.value.as_deref().unwrap_or(""));
            conn.execute("UPDATE job_selection SET selected=1 WHERE job_id=?1 AND job_item_id IN(SELECT id FROM job_items WHERE job_id=?1 AND source_path LIKE ?2 AND state='new')",params![request.job_id,prefix]).map_err(|e|e.to_string())?;
        }
        "maximum_safe" => {
            let limit = request
                .maximum_bytes
                .unwrap_or(storage_plan(cfg, &request.job_id)?.maximum_safe_bytes)
                .min(i64::MAX as u64) as i64;
            conn.execute("UPDATE job_selection SET selected=1 WHERE job_id=?1 AND job_item_id IN(SELECT id FROM(SELECT id,SUM(bytes)OVER(ORDER BY bytes,id)running FROM job_items WHERE job_id=?1 AND state='new')WHERE running<=?2)",params![request.job_id,limit]).map_err(|e|e.to_string())?;
        }
        _ => return Err("Modo de seleção desconhecido".into()),
    }
    let (selected_items,selected_bytes,pending_items,pending_bytes):(i64,i64,i64,i64)=conn.query_row("SELECT COALESCE(SUM(s.selected),0),COALESCE(SUM(CASE WHEN s.selected=1 THEN ji.bytes ELSE 0 END),0),COALESCE(SUM(s.selected=0),0),COALESCE(SUM(CASE WHEN s.selected=0 THEN ji.bytes ELSE 0 END),0) FROM job_selection s JOIN job_items ji ON ji.id=s.job_item_id WHERE s.job_id=?1 AND ji.state='new'",[&request.job_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;
    event(
        &conn,
        &request.job_id,
        "",
        "selection_updated",
        &format!(
            "Seleção {}: {} itens e {} bytes; {} itens pendentes",
            request.mode, selected_items, selected_bytes, pending_items
        ),
    );
    Ok(crate::models::SelectionResult {
        selected_items,
        selected_bytes: selected_bytes.max(0) as u64,
        pending_items,
        pending_bytes: pending_bytes.max(0) as u64,
    })
}

pub fn consolidate(cfg: &LibraryConfig, job: &str) -> Result<(), String> {
    consolidate_cancel(cfg, job, &crate::process::CancellationToken::default())
}
pub fn consolidate_cancel(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &crate::process::CancellationToken,
) -> Result<(), String> {
    let db_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let mut conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
    let source_id: String = conn
        .query_row("SELECT source_id FROM jobs WHERE id=?1", [job], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?;
    let plan = storage_plan(cfg, job)?;
    if !plan.can_consolidate {
        let error = format!(
            "Espaço insuficiente: faltam {} bytes para biblioteca e réplica",
            plan.missing_bytes
        );
        conn.execute("UPDATE jobs SET state='waiting_space',stage='space_check',interruption_reason=?2,finished_at=NULL,updated_at=?3 WHERE id=?1",params![job,error,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        event(&conn, job, &cfg.master_path, "waiting_space", &error);
        return Ok(());
    }
    conn.execute("UPDATE jobs SET state='consolidating',stage='copying',processed_items=0,processed_bytes=0,interruption_reason=NULL,started_at=COALESCE(started_at,?2),updated_at=?2 WHERE id=?1",params![job,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    let mut stmt=conn.prepare("SELECT p.id,p.path,p.filename,p.extension,p.media_type,p.bytes,p.modified_at,p.hash,p.status FROM pending_files p JOIN job_items ji ON ji.job_id=p.job_id AND ji.source_path=p.path LEFT JOIN job_selection s ON s.job_id=ji.job_id AND s.job_item_id=ji.id WHERE p.job_id=?1 AND (p.status='duplicate' OR (p.status IN('new','failed_copy','failed_backup') AND COALESCE(s.selected,1)=1)) ORDER BY p.id").map_err(|e|e.to_string())?;
    let rows = stmt
        .query_map([job], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    let total_items = rows.len() as i64;
    let total_bytes: i64 = rows.iter().map(|r| r.5).sum();
    conn.execute(
        "UPDATE jobs SET total_items=?2,total_bytes=?3 WHERE id=?1",
        params![job, total_items, total_bytes],
    )
    .ok();
    let started_clock = Instant::now();
    let mut last_measure = started_clock;
    let mut last_measured_bytes = 0i64;
    let mut moving_speed = 0.0f64;
    let mut processed_items = 0i64;
    let mut processed_bytes = 0i64;
    let mut imported = 0i64;
    let mut duplicate_count = 0i64;
    let mut failed = 0i64;
    let mut copy_ms = 0i64;
    let thumbnail_ms = 0i64;
    let backup_ms = 0i64;
    for (pid, path, filename, ext, kind, bytes, modified, known_hash, status) in rows {
        control_point(&conn, job)?;
        conn.execute(
            "UPDATE jobs SET current_file=?2,stage='copying',updated_at=?3 WHERE id=?1",
            params![job, path, Utc::now().to_rfc3339()],
        )
        .ok();
        let source = PathBuf::from(&path);
        let deferred_temp = Path::new(&cfg.master_path)
            .join(".lumina/temp")
            .join(job)
            .join(format!("{pid}.part"));
        let mut item_copy_ms = 0i64;
        let (hash, pre_copied) = match known_hash {
            Some(value) => (value, false),
            None => {
                conn.execute("UPDATE jobs SET stage='copying_and_identifying',current_file=?2,updated_at=?3 WHERE id=?1",params![job,path,Utc::now().to_rfc3339()]).ok();
                drop(conn);
                let deferred_copy_started = Instant::now();
                let _io = crate::resource::io(crate::resource::Priority::Background);
                let value = crate::storage::copy_hash_to_temp_verified(
                    &source,
                    &deferred_temp,
                    Some(cancel),
                )?;
                drop(_io);
                item_copy_ms += deferred_copy_started.elapsed().as_millis() as i64;
                conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
                conn.execute(
                    "UPDATE pending_files SET hash=?2 WHERE id=?1",
                    params![pid, value],
                )
                .map_err(|e| e.to_string())?;
                conn.execute("UPDATE job_items SET sha256=?3,temp_path=?4,current_stage='verification',updated_at=?5 WHERE job_id=?1 AND source_path=?2",params![job,path,value,deferred_temp.to_string_lossy(),Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                (value, true)
            }
        };
        copy_ms += item_copy_ms;
        if status == "duplicate" {
            duplicate_count += 1
        }
        let existing = conn
            .query_row("SELECT id FROM assets WHERE hash=?1", [&hash], |r| {
                r.get::<_, String>(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let asset_id = if let Some(id) = existing {
            if pre_copied {
                let _ = fs::remove_file(&deferred_temp);
            }
            id
        } else {
            let stored_metadata = conn.query_row(
                "SELECT captured_at,date_source,width,height,duration,camera,latitude,longitude FROM job_items WHERE job_id=?1 AND source_path=?2",
                params![job,path],
                |row| Ok((row.get::<_,Option<String>>(0)?,row.get::<_,Option<String>>(1)?,row.get::<_,Option<i64>>(2)?,row.get::<_,Option<i64>>(3)?,row.get::<_,Option<f64>>(4)?,row.get::<_,Option<String>>(5)?,row.get::<_,Option<f64>>(6)?,row.get::<_,Option<f64>>(7)?)),
            ).optional().map_err(|e|e.to_string())?;
            let (captured, date_source, width, height, duration, camera, lat, lng) =
                match stored_metadata {
                    Some((
                        Some(captured),
                        Some(date_source),
                        width,
                        height,
                        duration,
                        camera,
                        lat,
                        lng,
                    )) => (
                        captured,
                        date_source,
                        width,
                        height,
                        duration,
                        camera,
                        lat,
                        lng,
                    ),
                    _ => {
                        drop(conn);
                        let metadata = capture_metadata(&source, &modified, cancel);
                        conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
                        metadata
                    }
                };
            let parsed = DateTime::parse_from_rfc3339(&captured)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let dir = Path::new(&cfg.master_path)
                .join(parsed.format("%Y").to_string())
                .join(parsed.format("%m").to_string());
            let stored_paths:(Option<String>,Option<String>)=conn.query_row("SELECT destination_path,temp_path FROM job_items WHERE job_id=?1 AND source_path=?2",params![job,path],|row|Ok((row.get(0)?,row.get(1)?))).map_err(|e|e.to_string())?;
            let dest = match stored_paths.0 {
                Some(value) => PathBuf::from(value),
                None => crate::storage::safe_destination(&dir, &filename, &source, &hash)?,
            };
            let temp = stored_paths.1.map(PathBuf::from).unwrap_or_else(|| {
                Path::new(&cfg.master_path)
                    .join(".lumina/temp")
                    .join(job)
                    .join(format!("{pid}.part"))
            });
            conn.execute(
                "UPDATE job_items SET destination_path=COALESCE(destination_path,?3),temp_path=COALESCE(temp_path,?4),current_stage='copying',updated_at=?5 WHERE job_id=?1 AND source_path=?2",
                params![job,path,dest.to_string_lossy(),temp.to_string_lossy(),Utc::now().to_rfc3339()],
            ).map_err(|e|e.to_string())?;
            drop(conn);
            let copy_started = Instant::now();
            let _io = crate::resource::io(crate::resource::Priority::Background);
            let copy_result = if pre_copied {
                crate::storage::promote_preverified_temp(&temp, &dest, &hash)
            } else {
                crate::storage::copy_verified_via_staged(&source, &dest, &temp, &hash, |stage| {
                    if let Ok(stage_conn) = catalog::open(&db_path) {
                        let _=stage_conn.execute("UPDATE job_items SET current_stage=?3,updated_at=?4 WHERE job_id=?1 AND source_path=?2",params![job,path,stage,Utc::now().to_rfc3339()]);
                        let _ = stage_conn.execute(
                            "UPDATE jobs SET stage=?2,updated_at=?3 WHERE id=?1",
                            params![job, stage, Utc::now().to_rfc3339()],
                        );
                    }
                })
            };
            drop(_io);
            copy_ms += copy_started.elapsed().as_millis() as i64;
            conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
            if let Err(err) = copy_result {
                failed += 1;
                conn.execute(
                    "UPDATE pending_files SET status='failed_copy',error=?2 WHERE id=?1",
                    params![pid, err],
                )
                .ok();
                conn.execute("UPDATE job_items SET state='failed',last_error_kind='copy',last_error=?3,updated_at=?4 WHERE job_id=?1 AND source_path=?2",params![job,path,err,Utc::now().to_rfc3339()]).ok();
                event(&conn, job, &path, "failed", "Falha ao copiar/verificar");
                processed_items += 1;
                processed_bytes += bytes;
                conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,failed_count=?4 WHERE id=?1",params![job,processed_items,processed_bytes,failed]).ok();
                continue;
            }
            conn.execute(
                "UPDATE job_items SET temp_path=NULL,current_stage='cataloging',updated_at=?3 WHERE job_id=?1 AND source_path=?2",
                params![job,path,Utc::now().to_rfc3339()],
            ).map_err(|e|e.to_string())?;
            let id = Uuid::new_v4().to_string();
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,width,height,duration,camera,latitude,longitude,master_path,protection_state,created_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,'consolidated',?16)",params![id,hash,filename,kind,ext,captured,date_source,bytes,width,height,duration,camera,lat,lng,dest.to_string_lossy(),Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
            let format = crate::formats::descriptor(&ext);
            let (detected, matches) = crate::formats::detected_format(&dest, &ext);
            conn.execute("INSERT INTO asset_technical_metadata(asset_id,declared_extension,detected_format,family,support_level,extension_matches,metadata_supported,thumbnail_supported,preview_supported,enriched_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![id,ext,detected,format.family.as_str(),format.support.as_str(),matches,format.metadata,format.thumbnail,format.preview,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
            id
        };
        if status != "duplicate" {
            imported += 1;
        }
        conn.execute("INSERT OR IGNORE INTO occurrences(id,asset_id,source_id,path,seen_at)VALUES(?1,?2,?3,?4,?5)",params![Uuid::new_v4().to_string(),asset_id,source_id,path,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE pending_files SET status='consolidated',error=NULL WHERE id=?1",
            [pid],
        )
        .ok();
        conn.execute(
            "UPDATE job_items SET state='consolidated',current_stage='cataloged',last_error_kind=NULL,last_error=NULL,updated_at=?3 WHERE job_id=?1 AND source_path=?2",
            params![job,path,Utc::now().to_rfc3339()],
        ).ok();
        conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES(?1,?2,'','pending',?3)ON CONFLICT(asset_id)DO UPDATE SET state=CASE WHEN thumbnails.state='ready' THEN 'ready' ELSE 'pending' END,updated_at=excluded.updated_at",params![asset_id,crate::media::THUMBNAIL_VERSION,Utc::now().to_rfc3339()]).ok();
        conn.execute("INSERT INTO work_queue(job_id,asset_id,job_item_id,kind,state,created_at,updated_at)VALUES(?1,?2,(SELECT id FROM job_items WHERE job_id=?1 AND source_path=?3),'thumbnail','pending',?4,?4)ON CONFLICT(job_id,asset_id,kind)DO NOTHING",params![job,asset_id,path,Utc::now().to_rfc3339()]).ok();
        if cancel.is_cancelled() {
            conn.execute(
                "UPDATE jobs SET state='canceled',finished_at=?2,updated_at=?2 WHERE id=?1",
                params![job, Utc::now().to_rfc3339()],
            )
            .ok();
            return Err("JOB_CANCELED".into());
        }
        conn.execute("INSERT INTO work_queue(job_id,asset_id,job_item_id,kind,state,created_at,updated_at)VALUES(?1,?2,(SELECT id FROM job_items WHERE job_id=?1 AND source_path=?3),'backup','pending',?4,?4)ON CONFLICT(job_id,asset_id,kind)DO NOTHING",params![job,asset_id,path,Utc::now().to_rfc3339()]).ok();
        conn.execute("UPDATE assets SET protection_state=CASE WHEN protection_state='replica_verified' THEN protection_state ELSE 'consolidated' END WHERE id=?1",[&asset_id]).ok();
        conn.execute("UPDATE job_items SET current_stage='protection_pending',updated_at=?3 WHERE job_id=?1 AND source_path=?2",params![job,path,Utc::now().to_rfc3339()]).ok();
        processed_items += 1;
        processed_bytes += bytes;
        let interval = last_measure.elapsed().as_secs_f64().max(0.001);
        let instantaneous = (processed_bytes - last_measured_bytes).max(0) as f64 / interval;
        moving_speed = if moving_speed == 0.0 {
            instantaneous
        } else {
            moving_speed * 0.75 + instantaneous * 0.25
        };
        last_measure = Instant::now();
        last_measured_bytes = processed_bytes;
        let eta = if moving_speed > 0.0 {
            Some(((total_bytes - processed_bytes).max(0) as f64 / moving_speed).ceil() as i64)
        } else {
            None
        };
        if progress_due(processed_items, total_items) {
            conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,stage_processed_items=?2,stage_total_items=?8,stage_processed_bytes=?3,stage_total_bytes=?9,imported_count=?4,duplicate_count=?5,failed_count=?6,stage='copying',backup_state='pending',bytes_per_second=?10,estimated_seconds_remaining=?11,updated_at=?7 WHERE id=?1",params![job,processed_items,processed_bytes,imported,duplicate_count,failed,Utc::now().to_rfc3339(),total_items,total_bytes,moving_speed,eta]).ok();
            conn.execute(
                "UPDATE job_counters SET imported=?2,duplicates=?3,failed=?4 WHERE job_id=?1",
                params![job, imported, duplicate_count, failed],
            )
            .ok();
        }
    }
    conn.execute("UPDATE sources SET asset_count=(SELECT COUNT(*) FROM occurrences WHERE source_id=?1),last_scan=?2 WHERE id=?1",params![source_id,Utc::now().to_rfc3339()]).ok();
    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM job_items WHERE job_id=?1 AND state='new'",
            [job],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let final_state = if remaining > 0 {
        "batch_pending"
    } else {
        "protection_pending"
    };
    conn.execute("UPDATE jobs SET state=?2,stage=?2,current_file=NULL,processed_items=?3,total_items=?3,processed_bytes=?4,total_bytes=?4,finished_at=?5,updated_at=?5,library_state='verified',backup_state='pending' WHERE id=?1",params![job,final_state,total_items,total_bytes,Utc::now().to_rfc3339()]).ok();
    for (stage, duration) in [
        ("copy_and_verify", copy_ms),
        ("thumbnails", thumbnail_ms),
        ("backup_and_verify", backup_ms),
        (
            "consolidation_total",
            started_clock.elapsed().as_millis() as i64,
        ),
    ] {
        conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,?2,?3,?4,?5,?6)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,stage,duration,total_items,total_bytes,Utc::now().to_rfc3339()]).ok();
    }
    event(
        &conn,
        job,
        "",
        final_state,
        if remaining > 0 {
            "Lote consolidado; há mídias aguardando o próximo lote"
        } else {
            "Consolidação concluída; proteção aguardando na fila"
        },
    );
    conn.execute_batch("PRAGMA wal_checkpoint(FULL)").ok();
    Ok(())
}

pub fn process_thumbnail_queue(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &crate::process::CancellationToken,
) -> Result<(), String> {
    let db_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let started = Instant::now();
    let mut generated = 0i64;
    let mut failed = 0i64;
    loop {
        if cancel.is_cancelled() {
            break;
        }
        let conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
        let next=conn.query_row("SELECT q.id,a.id,a.master_path,a.extension,a.hash FROM work_queue q JOIN assets a ON a.id=q.asset_id WHERE q.kind='thumbnail' AND q.state='pending' ORDER BY q.priority DESC,q.id LIMIT 1",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?))).optional().map_err(|e|e.to_string())?;
        let Some((qid, asset, path, ext, hash)) = next else {
            break;
        };
        conn.execute("UPDATE work_queue SET state='processing',attempts=attempts+1,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).ok();
        drop(conn);
        let _io = crate::resource::io(crate::resource::Priority::Interactive);
        let result = crate::media::generate_thumbnail(
            Path::new(&path),
            &ext,
            &hash,
            &Path::new(&cfg.master_path).join(".lumina/cache"),
            cancel,
        );
        drop(_io);
        let conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
        match result {
            Ok(thumb) => {
                generated += 1;
                let file_bytes = thumb.metadata().map(|value| value.len()).unwrap_or(0);
                conn.execute("UPDATE thumbnails SET generator_version=?2,path=?3,file_bytes=?4,state='ready',last_error=NULL,updated_at=?5 WHERE asset_id=?1",params![asset,crate::media::THUMBNAIL_VERSION,thumb.to_string_lossy(),file_bytes,Utc::now().to_rfc3339()]).ok();
                conn.execute("UPDATE work_queue SET state='completed',last_error=NULL,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).ok();
            }
            Err(error) => {
                failed += 1;
                conn.execute(
                    "UPDATE work_queue SET state='failed',last_error=?2,updated_at=?3 WHERE id=?1",
                    params![qid, error, Utc::now().to_rfc3339()],
                )
                .ok();
                conn.execute("UPDATE thumbnails SET state='failed',last_error=?2,updated_at=?3 WHERE asset_id=?1",params![asset,error,Utc::now().to_rfc3339()]).ok();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'thumbnail_background',?2,?3,0,?4)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,recorded_at=excluded.recorded_at",params![job,started.elapsed().as_millis()as i64,generated,Utc::now().to_rfc3339()]).ok();
    conn.execute(
        "UPDATE job_counters SET thumbnailed=thumbnailed+?2,failed=failed+?3 WHERE job_id=?1",
        params![job, generated, failed],
    )
    .ok();
    Ok(())
}

pub fn queue_verification(cfg: &LibraryConfig) -> Result<String, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let job = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT OR IGNORE INTO sources(id,name,path,volume_label,available)VALUES('_lumina_maintenance','Manutenção da biblioteca','lumina://maintenance','internal',1)",[]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at,total_items,total_bytes)VALUES(?1,'_lumina_maintenance','lumina://verification','queued','verification',?2,?2,(SELECT COUNT(*) FROM backup_entries),(SELECT COALESCE(SUM(a.bytes),0) FROM backup_entries b JOIN assets a ON a.id=b.asset_id))",params![job,now]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO work_queue(job_id,asset_id,kind,state,created_at,updated_at)SELECT ?1,asset_id,'verification','pending',?2,?2 FROM backup_entries",params![job,now]).map_err(|e|e.to_string())?;
    Ok(job)
}
pub fn verify_job(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &crate::process::CancellationToken,
) -> Result<VerifyResult, String> {
    let db_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    conn.execute("UPDATE work_queue SET state='pending',updated_at=?2 WHERE job_id=?1 AND kind='verification' AND state='processing'",params![job,now]).map_err(|e|e.to_string())?;
    conn.execute("UPDATE jobs SET state='protecting',stage='verification',processed_items=0,processed_bytes=0,updated_at=?2 WHERE id=?1",params![job,now]).map_err(|e|e.to_string())?;
    let rows = {
        let mut statement=conn.prepare("SELECT q.id,b.asset_id,b.path,b.hash,a.bytes FROM work_queue q JOIN backup_entries b ON b.asset_id=q.asset_id JOIN assets a ON a.id=b.asset_id WHERE q.job_id=?1 AND q.kind='verification' AND q.state IN('pending','failed') ORDER BY q.id").map_err(|e|e.to_string())?;
        let values = statement
            .query_map([job], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let total = rows.len() as i64;
    let total_bytes = rows.iter().map(|row| row.4).sum::<i64>();
    let mut checked = 0;
    let mut checked_bytes = 0;
    let mut errors = 0;
    for (qid, asset, path, hash, bytes) in rows {
        control_point(&conn, job)?;
        if cancel.is_cancelled() {
            return Err("JOB_CANCELED".into());
        }
        conn.execute("UPDATE work_queue SET state='processing',attempts=attempts+1,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        let _io = crate::resource::io(crate::resource::Priority::Background);
        let verified = crate::backup::verify(Path::new(&path), &hash);
        drop(_io);
        checked += 1;
        checked_bytes += bytes;
        if verified {
            conn.execute(
                "UPDATE backup_entries SET state='verified',verified_at=?2 WHERE asset_id=?1",
                params![asset, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE work_queue SET state='completed',last_error=NULL,updated_at=?2 WHERE id=?1",
                params![qid, Utc::now().to_rfc3339()],
            )
            .map_err(|e| e.to_string())?;
        } else {
            errors += 1;
            conn.execute(
                "UPDATE backup_entries SET state='error' WHERE asset_id=?1",
                [&asset],
            )
            .map_err(|e| e.to_string())?;
            conn.execute(
                "UPDATE assets SET protection_state='error' WHERE id=?1",
                [&asset],
            )
            .map_err(|e| e.to_string())?;
            conn.execute("UPDATE work_queue SET state='failed',last_error='Hash da réplica não confere',updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        }
        if progress_due(checked, total) {
            conn.execute("UPDATE jobs SET processed_items=?2,total_items=?3,processed_bytes=?4,total_bytes=?5,current_file=?6,updated_at=?7 WHERE id=?1",params![job,checked,total,checked_bytes,total_bytes,path,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        }
    }
    let state = if errors == 0 { "completed" } else { "failed" };
    conn.execute("UPDATE jobs SET state=?2,stage=?3,current_file=NULL,finished_at=?4,updated_at=?4,failed_count=?5 WHERE id=?1",params![job,state,if errors==0{"completed"}else{"verification_error"},Utc::now().to_rfc3339(),errors]).map_err(|e|e.to_string())?;
    Ok(VerifyResult { checked, errors })
}
#[cfg(test)]
pub fn verify(cfg: &LibraryConfig) -> Result<VerifyResult, String> {
    let job = queue_verification(cfg)?;
    verify_job(cfg, &job, &crate::process::CancellationToken::default())
}

pub fn queue_format_enrichment(cfg: &LibraryConfig) -> Result<String, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    if let Some(job)=conn.query_row("SELECT id FROM jobs WHERE stage='technical_enrichment' AND state IN('queued','analyzing','paused','interrupted') ORDER BY created_at DESC LIMIT 1",[],|row|row.get::<_,String>(0)).optional().map_err(|e|e.to_string())?{return Ok(job)}
    let missing:i64=conn.query_row("SELECT COUNT(*) FROM assets a LEFT JOIN asset_technical_metadata t ON t.asset_id=a.id WHERE t.asset_id IS NULL OR t.inventory_state!='complete' OR (a.media_type='video' AND (t.codec IS NULL OR t.container IS NULL))",[],|row|row.get(0)).map_err(|error|error.to_string())?;
    if missing == 0 {
        if let Some(job)=conn.query_row("SELECT id FROM jobs WHERE source_id='_lumina_maintenance' ORDER BY created_at DESC LIMIT 1",[],|row|row.get::<_,String>(0)).optional().map_err(|error|error.to_string())? { return Ok(job); }
    }
    let job = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute("INSERT OR IGNORE INTO sources(id,name,path,volume_label,available)VALUES('_lumina_maintenance','Manutenção da biblioteca','lumina://maintenance','internal',1)",[]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at,total_items,total_bytes)VALUES(?1,'_lumina_maintenance','lumina://format-enrichment','queued','technical_enrichment',?2,?2,?3,(SELECT COALESCE(SUM(a.bytes),0) FROM assets a LEFT JOIN asset_technical_metadata t ON t.asset_id=a.id WHERE t.asset_id IS NULL OR t.inventory_state!='complete' OR (a.media_type='video' AND (t.codec IS NULL OR t.container IS NULL))))",params![job,now,missing]).map_err(|e|e.to_string())?;
    conn.execute("INSERT INTO work_queue(job_id,asset_id,kind,state,priority,created_at,updated_at)SELECT ?1,a.id,'technical_metadata','pending',-10,?2,?2 FROM assets a LEFT JOIN asset_technical_metadata t ON t.asset_id=a.id WHERE t.asset_id IS NULL OR t.inventory_state!='complete' OR (a.media_type='video' AND (t.codec IS NULL OR t.container IS NULL))",params![job,now]).map_err(|e|e.to_string())?;
    Ok(job)
}

pub fn enrich_formats_job(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &crate::process::CancellationToken,
) -> Result<(), String> {
    let db_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    conn.execute("UPDATE work_queue SET state='pending',updated_at=?2 WHERE job_id=?1 AND kind='technical_metadata' AND state='processing'",params![job,now]).map_err(|e|e.to_string())?;
    conn.execute("UPDATE jobs SET state='analyzing',stage='technical_enrichment',processed_items=0,processed_bytes=0,updated_at=?2 WHERE id=?1",params![job,now]).map_err(|e|e.to_string())?;
    let rows = {
        let mut statement=conn.prepare("SELECT q.id,a.id,a.master_path,a.extension,a.bytes FROM work_queue q JOIN assets a ON a.id=q.asset_id WHERE q.job_id=?1 AND q.kind='technical_metadata' AND q.state IN('pending','failed') ORDER BY q.id").map_err(|e|e.to_string())?;
        let values = statement
            .query_map([job], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let total = rows.len() as i64;
    let total_bytes = rows.iter().map(|row| row.4).sum::<i64>();
    let photo_paths = rows
        .iter()
        .filter(|row| {
            crate::formats::descriptor(&row.3).family != crate::formats::MediaFamily::Video
        })
        .map(|row| PathBuf::from(&row.2))
        .collect::<Vec<_>>();
    let photo_metadata = technical_photo_batches(&photo_paths, cancel);
    let mut done = 0;
    let mut done_bytes = 0;
    for (qid, asset, path, extension, bytes) in rows {
        control_point(&conn, job)?;
        if cancel.is_cancelled() {
            return Err("JOB_CANCELED".into());
        }
        conn.execute("UPDATE work_queue SET state='processing',attempts=attempts+1,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        let descriptor = crate::formats::descriptor(&extension);
        let (detected, matches) = crate::formats::detected_format(Path::new(&path), &extension);
        let mut codec = None;
        let mut container = None;
        let mut audio_codec: Option<String> = None;
        let mut frame_rate: Option<f64> = None;
        let mut bitrate: Option<i64> = None;
        let mut pixel_format: Option<String> = None;
        let mut lens: Option<String> = None;
        let mut iso: Option<i64> = None;
        let mut aperture: Option<f64> = None;
        let mut exposure: Option<String> = None;
        let mut focal_length: Option<f64> = None;
        let mut orientation: Option<i64> = None;
        let mut color_profile: Option<String> = None;
        let mut preview_available: Option<bool> = None;
        let mut inventory_error: Option<String> = None;
        if descriptor.family == crate::formats::MediaFamily::Video {
            let spec = crate::process::ProcessSpec::new("FFprobe", "ffprobe")
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "stream=codec_type,codec_name,width,height,r_frame_rate,bit_rate,pix_fmt:format=format_name,bit_rate,duration",
                    "-of",
                    "json",
                    path.as_str(),
                ])
                .timeout(std::time::Duration::from_secs(30))
                .logical("FFprobe technical inventory");
            if let Ok(result) = crate::process::run(spec, cancel) {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&result.stdout) {
                    codec = json
                        .get("streams")
                        .and_then(|v| v.as_array())
                        .and_then(|items| {
                            items.iter().find(|v| {
                                v.get("codec_type").and_then(|x| x.as_str()) == Some("video")
                            })
                        })
                        .and_then(|v| v.get("codec_name"))
                        .and_then(|value| value.as_str())
                        .map(str::to_string);
                    let video = json
                        .get("streams")
                        .and_then(|v| v.as_array())
                        .and_then(|items| {
                            items.iter().find(|v| {
                                v.get("codec_type").and_then(|x| x.as_str()) == Some("video")
                            })
                        });
                    audio_codec = json
                        .get("streams")
                        .and_then(|v| v.as_array())
                        .and_then(|items| {
                            items.iter().find(|v| {
                                v.get("codec_type").and_then(|x| x.as_str()) == Some("audio")
                            })
                        })
                        .and_then(|v| v.get("codec_name"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    pixel_format = video
                        .and_then(|v| v.get("pix_fmt"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    bitrate = video
                        .and_then(|v| v.get("bit_rate"))
                        .and_then(|v| v.as_str())
                        .and_then(|v| v.parse().ok())
                        .or_else(|| {
                            json.pointer("/format/bit_rate")
                                .and_then(|v| v.as_str())
                                .and_then(|v| v.parse().ok())
                        });
                    frame_rate = video
                        .and_then(|v| v.get("r_frame_rate"))
                        .and_then(|v| v.as_str())
                        .and_then(|v| {
                            let mut p = v.split('/');
                            let a = p.next()?.parse::<f64>().ok()?;
                            let b = p.next().unwrap_or("1").parse::<f64>().ok()?;
                            if b > 0.0 {
                                Some(a / b)
                            } else {
                                None
                            }
                        });
                    container = json
                        .pointer("/format/format_name")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                } else {
                    inventory_error = Some("FFprobe retornou dados técnicos inválidos".into());
                }
            }
        } else {
            match photo_metadata.get(&cache_key(Path::new(&path))) {
                Some(value) => {
                    lens = value
                        .get("LensModel")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    iso = value.get("ISO").and_then(|v| v.as_i64());
                    aperture = value.get("FNumber").and_then(|v| v.as_f64());
                    exposure = value.get("ExposureTime").map(|v| v.to_string());
                    focal_length = value.get("FocalLength").and_then(|v| v.as_f64());
                    orientation = value.get("Orientation").and_then(|v| v.as_i64());
                    color_profile = value
                        .get("ColorSpace")
                        .map(|v| v.to_string().trim_matches('"').to_string());
                    preview_available = value
                        .get("PreviewImageLength")
                        .and_then(|v| v.as_i64())
                        .map(|v| v > 0);
                }
                None => inventory_error = Some("Metadados técnicos não retornados".into()),
            }
        }
        let inventory_state = if inventory_error.is_none()
            && (descriptor.family != crate::formats::MediaFamily::Video
                || (codec.is_some() && container.is_some()))
        {
            "complete"
        } else {
            "partial"
        };
        conn.execute("INSERT INTO asset_technical_metadata(asset_id,declared_extension,detected_format,family,container,codec,audio_codec,frame_rate,bitrate,pixel_format,lens,iso,aperture,exposure,focal_length,orientation,color_profile,preview_available,inventory_state,inventory_error,support_level,extension_matches,metadata_supported,thumbnail_supported,preview_supported,enriched_at)VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26)ON CONFLICT(asset_id)DO UPDATE SET declared_extension=excluded.declared_extension,detected_format=excluded.detected_format,family=excluded.family,container=excluded.container,codec=excluded.codec,audio_codec=excluded.audio_codec,frame_rate=excluded.frame_rate,bitrate=excluded.bitrate,pixel_format=excluded.pixel_format,lens=excluded.lens,iso=excluded.iso,aperture=excluded.aperture,exposure=excluded.exposure,focal_length=excluded.focal_length,orientation=excluded.orientation,color_profile=excluded.color_profile,preview_available=excluded.preview_available,inventory_state=excluded.inventory_state,inventory_error=excluded.inventory_error,support_level=excluded.support_level,extension_matches=excluded.extension_matches,metadata_supported=excluded.metadata_supported,thumbnail_supported=excluded.thumbnail_supported,preview_supported=excluded.preview_supported,enriched_at=excluded.enriched_at",params![asset,extension,detected,descriptor.family.as_str(),container,codec,audio_codec,frame_rate,bitrate,pixel_format,lens,iso,aperture,exposure,focal_length,orientation,color_profile,preview_available,inventory_state,inventory_error,descriptor.support.as_str(),matches,descriptor.metadata,descriptor.thumbnail,descriptor.preview,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        conn.execute(
            "UPDATE work_queue SET state='completed',last_error=NULL,updated_at=?2 WHERE id=?1",
            params![qid, Utc::now().to_rfc3339()],
        )
        .map_err(|e| e.to_string())?;
        done += 1;
        done_bytes += bytes;
        if progress_due(done, total) {
            conn.execute("UPDATE jobs SET processed_items=?2,total_items=?3,processed_bytes=?4,total_bytes=?5,current_file=?6,updated_at=?7 WHERE id=?1",params![job,done,total,done_bytes,total_bytes,path,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        }
    }
    conn.execute("UPDATE jobs SET state='completed',stage='completed',current_file=NULL,finished_at=?2,updated_at=?2 WHERE id=?1",params![job,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    Ok(())
}

pub fn protection_stats(
    cfg: &LibraryConfig,
    job: Option<&str>,
) -> Result<crate::models::ProtectionQueueStats, String> {
    let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        .map_err(|e| e.to_string())?;
    let filter = if job.is_some() {
        " WHERE q.kind='backup' AND q.job_id=?1"
    } else {
        " WHERE q.kind='backup' AND (?1 IS NULL OR 1=1)"
    };
    let sql=format!("SELECT COALESCE(SUM(q.state='pending'),0),COALESCE(SUM(q.state='processing'),0),COALESCE(SUM(q.state='completed'),0),COALESCE(SUM(q.state='failed'),0),COALESCE(SUM(CASE WHEN q.state IN('pending','failed') THEN a.bytes ELSE 0 END),0) FROM work_queue q JOIN assets a ON a.id=q.asset_id{filter}");
    let value = job.map(str::to_string);
    conn.query_row(&sql, [value], |r| {
        Ok(crate::models::ProtectionQueueStats {
            pending: r.get(0)?,
            processing: r.get(1)?,
            completed: r.get(2)?,
            failed: r.get(3)?,
            pending_bytes: r.get(4)?,
        })
    })
    .map_err(|e| e.to_string())
}

fn write_backup_manifest(conn: &rusqlite::Connection, cfg: &LibraryConfig) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let mut statement = conn.prepare("SELECT asset_id,hash,path,verified_at FROM backup_entries WHERE state='verified' ORDER BY asset_id").map_err(|e|e.to_string())?;
    let entries = statement.query_map([],|row|Ok(serde_json::json!({"assetId":row.get::<_,String>(0)?,"sha256":row.get::<_,String>(1)?,"backupPath":row.get::<_,String>(2)?,"verifiedAt":row.get::<_,Option<String>>(3)?}))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let payload = entries
        .iter()
        .map(|entry| serde_json::to_string(entry).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let checksum = format!("{:x}", Sha256::digest(payload.as_bytes()));
    let header = serde_json::json!({"type":"lumina-manifest","schemaVersion":1,"generatedAt":Utc::now().to_rfc3339(),"entries":entries.len(),"payloadSha256":checksum});
    let contents = if payload.is_empty() {
        format!("{}\n", header)
    } else {
        format!("{}\n{}\n", header, payload)
    };
    crate::storage::atomic_write(
        &Path::new(&cfg.backup_path).join("manifest.jsonl"),
        contents.as_bytes(),
    )
}

pub fn protect_job(
    cfg: &LibraryConfig,
    job: &str,
    cancel: &crate::process::CancellationToken,
) -> Result<(), String> {
    let protection_started = Instant::now();
    let db_path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
    let mut conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
    let pending_bytes:i64=conn.query_row("SELECT COALESCE(SUM(a.bytes),0) FROM work_queue q JOIN assets a ON a.id=q.asset_id WHERE q.job_id=?1 AND q.kind='backup' AND q.state IN('pending','failed')",[job],|r|r.get(0)).map_err(|e|e.to_string())?;
    let available = fs2::available_space(Path::new(&cfg.backup_path)).map_err(|e| e.to_string())?;
    if pending_bytes.max(0) as u64 > available {
        let reason = format!(
            "A réplica precisa de {} bytes e possui {} bytes livres",
            pending_bytes, available
        );
        conn.execute("UPDATE jobs SET state='waiting_backup_space',stage='backup_space_check',backup_state='pending',interruption_reason=?2,updated_at=?3 WHERE id=?1",params![job,reason,Utc::now().to_rfc3339()]).ok();
        return Ok(());
    }
    conn.execute("UPDATE jobs SET state='protecting',stage='backup',backup_state='copying',interruption_reason=NULL,processed_items=0,processed_bytes=0,total_items=(SELECT COUNT(*) FROM work_queue WHERE job_id=?1 AND kind='backup' AND state IN('pending','failed')),total_bytes=?2,updated_at=?3 WHERE id=?1",params![job,pending_bytes,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
    let rows = {
        let mut s=conn.prepare("SELECT q.id,a.id,a.master_path,a.hash,a.bytes FROM work_queue q JOIN assets a ON a.id=q.asset_id WHERE q.job_id=?1 AND q.kind='backup' AND q.state IN('pending','failed') ORDER BY q.id").map_err(|e|e.to_string())?;
        let values = s
            .query_map([job], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        values
    };
    let total = rows.len() as i64;
    let mut done = 0i64;
    let mut done_bytes = 0i64;
    let mut failures = 0i64;
    let mut first_failure: Option<String> = None;
    for (qid, asset, master, hash, bytes) in rows {
        control_point(&conn, job)?;
        if cancel.is_cancelled() {
            return Err("JOB_CANCELED".into());
        }
        let rel = Path::new(&master)
            .strip_prefix(&cfg.master_path)
            .unwrap_or_else(|_| {
                Path::new(&master)
                    .file_name()
                    .map(Path::new)
                    .unwrap_or(Path::new("media"))
            });
        let destination = Path::new(&cfg.backup_path).join("originais").join(rel);
        conn.execute("UPDATE work_queue SET state='processing',attempts=attempts+1,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).ok();
        drop(conn);
        let _io = crate::resource::io(crate::resource::Priority::Background);
        let result = crate::backup::replicate(Path::new(&master), &destination, &hash);
        drop(_io);
        conn = catalog::open(&db_path).map_err(|e| e.to_string())?;
        match result {
            Ok(_) => {
                conn.execute("INSERT INTO backup_entries(asset_id,path,hash,verified_at,state)VALUES(?1,?2,?3,?4,'verified')ON CONFLICT(asset_id)DO UPDATE SET path=excluded.path,hash=excluded.hash,verified_at=excluded.verified_at,state='verified'",params![asset,destination.to_string_lossy(),hash,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
                conn.execute(
                    "UPDATE assets SET protection_state='replica_verified' WHERE id=?1",
                    [&asset],
                )
                .map_err(|e| e.to_string())?;
                conn.execute("UPDATE work_queue SET state='completed',last_error=NULL,updated_at=?2 WHERE id=?1",params![qid,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
            }
            Err(error) => {
                failures += 1;
                if first_failure.is_none() {
                    first_failure = Some(error.clone());
                }
                conn.execute(
                    "UPDATE work_queue SET state='failed',last_error=?2,updated_at=?3 WHERE id=?1",
                    params![qid, error, Utc::now().to_rfc3339()],
                )
                .map_err(|e| e.to_string())?;
                conn.execute(
                    "UPDATE assets SET protection_state='error' WHERE id=?1",
                    [&asset],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        done += 1;
        done_bytes += bytes;
        if progress_due(done, total) {
            conn.execute("UPDATE jobs SET processed_items=?2,processed_bytes=?3,stage_processed_items=?2,stage_total_items=?4,stage_processed_bytes=?3,stage_total_bytes=?5,current_file=?6,updated_at=?7 WHERE id=?1",params![job,done,done_bytes,total,pending_bytes,master,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        }
    }
    let mut final_error = first_failure.map(|error| format!("Falha ao replicar mídia: {error}"));
    if failures == 0 {
        if let Err(error) = write_backup_manifest(&conn, cfg) {
            failures += 1;
            final_error = Some(format!("Falha ao gravar manifesto verificado: {error}"));
        }
    }
    let state = if failures == 0 {
        "completed"
    } else {
        "backup_error"
    };
    let backup = if failures == 0 { "verified" } else { "error" };
    conn.execute("UPDATE jobs SET state=?2,stage=?2,backup_state=?3,current_file=NULL,finished_at=?4,updated_at=?4,interruption_reason=?5 WHERE id=?1",params![job,state,backup,Utc::now().to_rfc3339(),final_error]).map_err(|e|e.to_string())?;
    if final_error.is_some() {
        conn.execute("UPDATE assets SET protection_state='error' WHERE id IN(SELECT asset_id FROM work_queue WHERE job_id=?1 AND kind='backup')",[job]).map_err(|e|e.to_string())?;
    }
    conn.execute("INSERT INTO job_metrics(job_id,stage,duration_ms,items,bytes,recorded_at)VALUES(?1,'backup_and_verify',?2,?3,?4,?5)ON CONFLICT(job_id,stage)DO UPDATE SET duration_ms=excluded.duration_ms,items=excluded.items,bytes=excluded.bytes,recorded_at=excluded.recorded_at",params![job,protection_started.elapsed().as_millis()as i64,total,pending_bytes,Utc::now().to_rfc3339()]).ok();
    if failures == 0 {
        conn.execute_batch("PRAGMA wal_checkpoint(FULL)")
            .map_err(|e| e.to_string())?;
        if let Err(error) = catalog::snapshot(
            &db_path,
            &Path::new(&cfg.backup_path).join(".lumina/catalog.sqlite"),
        ) {
            let reason = format!("Falha ao criar snapshot consistente do catálogo: {error}");
            conn.execute("UPDATE jobs SET state='backup_error',stage='backup_error',backup_state='error',interruption_reason=?2,updated_at=?3 WHERE id=?1",params![job,reason,Utc::now().to_rfc3339()]).ok();
            conn.execute("UPDATE assets SET protection_state='error' WHERE id IN(SELECT asset_id FROM work_queue WHERE job_id=?1 AND kind='backup')",[job]).ok();
            return Err(reason);
        }
    }
    event(
        &conn,
        job,
        "",
        state,
        if failures == 0 {
            "Proteção concluída"
        } else {
            "Proteção concluída com falhas"
        },
    );
    if let Some(error) = final_error {
        Err(error)
    } else {
        Ok(())
    }
}

pub fn grouped_sources(conn: &rusqlite::Connection, asset: &str) -> Vec<String> {
    let mut s=match conn.prepare("SELECT DISTINCT s.name FROM sources s JOIN occurrences o ON o.source_id=s.id WHERE o.asset_id=?1"){Ok(v)=>v,Err(_)=>return vec![]};
    let result = s
        .query_map([asset], |r| r.get(0))
        .map(|rows| rows.filter_map(Result::ok).collect())
        .unwrap_or_default();
    result
}
pub fn grouped_tags(conn: &rusqlite::Connection, asset: &str) -> Vec<String> {
    let mut s = match conn
        .prepare("SELECT t.name FROM tags t JOIN asset_tags a ON a.tag_id=t.id WHERE a.asset_id=?1")
    {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let result = s
        .query_map([asset], |r| r.get(0))
        .map(|rows| rows.filter_map(Result::ok).collect())
        .unwrap_or_default();
    result
}
pub fn duplicate_occurrences(conn: &rusqlite::Connection, asset: &str) -> Vec<Occurrence> {
    let mut s=match conn.prepare("SELECT s.name,o.path FROM active_occurrences o JOIN sources s ON s.id=o.source_id WHERE o.asset_id=?1"){Ok(v)=>v,Err(_)=>return vec![]};
    let result = s
        .query_map([asset], |r| {
            Ok(Occurrence {
                source: r.get(0)?,
                path: r.get(1)?,
            })
        })
        .map(|rows| rows.filter_map(Result::ok).collect())
        .unwrap_or_default();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_persistence_is_batched_for_large_jobs() {
        let writes = (1..=1_500)
            .filter(|done| progress_due(*done, 1_500))
            .count();
        assert_eq!(writes, 188);
        assert!(writes < 200);
    }

    fn test_image(path: &Path, color: [u8; 3]) {
        image::ImageBuffer::<image::Rgb<u8>, _>::from_pixel(8, 8, image::Rgb(color))
            .save_with_format(path, image::ImageFormat::Jpeg)
            .unwrap()
    }

    fn fixture() -> (PathBuf, LibraryConfig) {
        let root = std::env::temp_dir().join(format!("lumina-pipeline-{}", Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(master.join(".lumina")).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: Uuid::new_v4().to_string(),
            name: "Teste".into(),
            master_path: master.to_string_lossy().into_owned(),
            backup_path: backup.to_string_lossy().into_owned(),
            created_at: Utc::now().to_rfc3339(),
        };
        catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        (root, cfg)
    }

    #[test]
    fn hash_is_stable_and_copy_is_verified() {
        let root = std::env::temp_dir().join(format!("lumina-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("origem.jpg");
        let destination = root.join("destino").join("copia.jpg");
        fs::write(&source, b"conteudo de teste do Lumina").unwrap();
        let hash = hash_file(&source).unwrap();
        copy_verified(&source, &destination, &hash).unwrap();
        assert_eq!(hash, hash_file(&destination).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_hash_never_promotes_temporary_copy() {
        let root = std::env::temp_dir().join(format!("lumina-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("origem.jpg");
        let destination = root.join("destino.jpg");
        fs::write(&source, b"arquivo original").unwrap();
        assert!(copy_verified(&source, &destination, "hash-incorreto").is_err());
        assert!(!destination.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn complete_pipeline_deduplicates_consolidates_backs_up_and_reimports() {
        let (root, cfg) = fixture();
        let source = root.join("camera");
        fs::create_dir_all(source.join("DCIM")).unwrap();
        test_image(&source.join("DCIM/IMG_0001.jpg"), [10, 20, 30]);
        let bytes = fs::read(source.join("DCIM/IMG_0001.jpg")).unwrap();
        fs::write(source.join("DCIM/copia-renomeada.jpg"), &bytes).unwrap();
        fs::write(source.join("leia-me.txt"), b"nao e midia").unwrap();

        let first = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        assert_eq!(first.discovered, 2);
        assert_eq!(first.new_files, 1);
        assert_eq!(first.duplicates, 1);
        assert_eq!(first.excluded, 1);
        consolidate(&cfg, &first.job_id).unwrap();
        process_thumbnail_queue(
            &cfg,
            &first.job_id,
            &crate::process::CancellationToken::default(),
        )
        .unwrap();
        let progress = job_progress(&cfg, &first.job_id).unwrap();
        assert_eq!(progress.state, "protection_pending");
        assert_eq!(progress.processed_items, progress.total_items);
        assert_eq!(progress.processed_bytes, progress.total_bytes);
        protect_job(
            &cfg,
            &first.job_id,
            &crate::process::CancellationToken::default(),
        )
        .unwrap();
        assert_eq!(
            job_progress(&cfg, &first.job_id).unwrap().state,
            "completed"
        );

        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM occurrences", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        let master: String = conn
            .query_row("SELECT master_path FROM assets", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fs::read(&master).unwrap(), bytes);
        assert_eq!(fs::read(source.join("DCIM/IMG_0001.jpg")).unwrap(), bytes);
        let result = verify(&cfg).unwrap();
        assert_eq!(result.checked, 1);
        assert_eq!(result.errors, 0);
        assert!(Path::new(&cfg.backup_path).join("manifest.jsonl").exists());
        assert!(Path::new(&cfg.backup_path)
            .join(".lumina/catalog.sqlite")
            .exists());
        let thumbnail: String = conn
            .query_row("SELECT path FROM thumbnails", [], |r| r.get(0))
            .unwrap();
        assert!(Path::new(&thumbnail).exists());
        assert_eq!(crate::media::clear_cache(&cfg).unwrap(), 1);
        assert!(!Path::new(&thumbnail).exists());
        let rebuilt = crate::media::rebuild_cache(&cfg).unwrap();
        assert_eq!((rebuilt.generated, rebuilt.failed), (1, 0));
        let rebuilt_path: String = conn
            .query_row("SELECT path FROM thumbnails", [], |r| r.get(0))
            .unwrap();
        assert!(Path::new(&rebuilt_path).exists());

        let second = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        assert_eq!(second.new_files, 0);
        assert_eq!(second.duplicates, 2);
        consolidate(&cfg, &second.job_id).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM occurrences", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_failure_never_reports_the_asset_as_protected() {
        let (root, cfg) = fixture();
        let source = root.join("camera-failure");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("IMG_0001.jpg"), [4, 5, 6]);
        let summary = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        consolidate(&cfg, &summary.job_id).unwrap();
        fs::remove_dir_all(&cfg.backup_path).unwrap();
        fs::write(&cfg.backup_path, b"not a directory").unwrap();
        assert!(protect_job(
            &cfg,
            &summary.job_id,
            &crate::process::CancellationToken::default()
        )
        .is_err());
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM assets WHERE protection_state='replica_verified'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        assert_ne!(
            conn.query_row(
                "SELECT state FROM jobs WHERE id=?1",
                [&summary.job_id],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            "completed"
        );
        drop(conn);
        fs::remove_file(&cfg.backup_path).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn verification_job_resumes_from_its_persistent_queue_after_cancel() {
        let (root, cfg) = fixture();
        let replica = root.join("backup/asset.jpg");
        fs::write(&replica, b"verified bytes").unwrap();
        let hash = hash_file(&replica).unwrap();
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,protection_state,created_at)VALUES('asset',?1,'asset.jpg','photo','jpg',?2,'file',14,'master.jpg','replica_verified',?2)",params![hash,now]).unwrap();
        conn.execute("INSERT INTO backup_entries(asset_id,path,hash,verified_at,state)VALUES('asset',?1,?2,?3,'verified')",params![replica.to_string_lossy(),hash,now]).unwrap();
        drop(conn);
        let job = queue_verification(&cfg).unwrap();
        let canceled = crate::process::CancellationToken::default();
        canceled.cancel();
        assert!(matches!(verify_job(&cfg, &job, &canceled), Err(error) if error == "JOB_CANCELED"));
        let result = verify_job(&cfg, &job, &crate::process::CancellationToken::default()).unwrap();
        assert_eq!((result.checked, result.errors), (1, 0));
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT state FROM work_queue WHERE job_id=?1",
                [job],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            "completed"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn analysis_enumerates_the_physical_source_once() {
        let (root, cfg) = fixture();
        let source = root.join("single-walk");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("one.jpg"), [1, 2, 3]);
        test_image(&source.join("two.jpg"), [3, 2, 1]);
        let summary = analyze(&cfg, source.to_str().unwrap(), "single walk").unwrap();
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT items FROM job_metrics WHERE job_id=?1 AND stage='inventory_walks'",
                [summary.job_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn partial_batches_preserve_the_remaining_analysis() {
        let (root, cfg) = fixture();
        let source = root.join("partial-camera");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("one.jpg"), [1, 2, 3]);
        test_image(&source.join("two.jpg"), [4, 5, 6]);
        test_image(&source.join("three.jpg"), [7, 8, 9]);
        let summary = analyze(&cfg, source.to_str().unwrap(), "Partial").unwrap();
        assert_eq!(summary.new_files, 3);
        let smallest = fs::metadata(source.join("one.jpg")).unwrap().len();
        let selection = apply_selection(
            &cfg,
            &crate::models::SelectionRequest {
                job_id: summary.job_id.clone(),
                mode: "maximum_safe".into(),
                value: None,
                maximum_bytes: Some(smallest),
            },
        )
        .unwrap();
        assert_eq!(selection.selected_items, 1);
        assert_eq!(selection.pending_items, 2);
        consolidate(&cfg, &summary.job_id).unwrap();
        assert_eq!(
            job_progress(&cfg, &summary.job_id).unwrap().state,
            "batch_pending"
        );
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row("SELECT COUNT(*)FROM assets", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(conn);
        apply_selection(
            &cfg,
            &crate::models::SelectionRequest {
                job_id: summary.job_id.clone(),
                mode: "all".into(),
                value: None,
                maximum_bytes: None,
            },
        )
        .unwrap();
        consolidate(&cfg, &summary.job_id).unwrap();
        assert_eq!(
            job_progress(&cfg, &summary.job_id).unwrap().state,
            "protection_pending"
        );
        protect_job(
            &cfg,
            &summary.job_id,
            &crate::process::CancellationToken::default(),
        )
        .unwrap();
        let stats = protection_stats(&cfg, Some(&summary.job_id)).unwrap();
        assert_eq!(stats.completed, 3);
        assert_eq!(stats.pending, 0);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn adaptive_hashing_uses_a_bounded_pool_and_returns_every_hash() {
        let (root, cfg) = fixture();
        let source = root.join("hash-pool");
        fs::create_dir_all(&source).unwrap();
        let paths = (0..16)
            .map(|i| {
                let p = source.join(format!("{i}.jpg"));
                fs::write(&p, vec![i as u8; 4096]).unwrap();
                p
            })
            .collect::<Vec<_>>();
        let catalog = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
        let conn = catalog::open(&catalog).unwrap();
        conn.execute("INSERT INTO sources(id,name,path,volume_label)VALUES('pool-source','pool','pool','pool')",[]).unwrap();
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('pool-job','pool-source','pool','analyzing',?1,?1)",[Utc::now().to_rfc3339()]).unwrap();
        drop(conn);
        let (results, workers, _) = hash_files_adaptive(
            paths,
            &catalog,
            "pool-job",
            16 * 4096,
            &crate::process::CancellationToken::default(),
        );
        assert_eq!(results.len(), 16);
        assert!((2..=4).contains(&workers));
        assert!(results.values().all(|x| x.is_ok()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unique_sizes_defer_hash_until_verified_copy() {
        let (root, cfg) = fixture();
        let source = root.join("unique-sizes");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("one.jpg"), [1, 2, 3]);
        image::ImageBuffer::<image::Rgb<u8>, _>::from_pixel(31, 17, image::Rgb([4, 5, 6]))
            .save_with_format(source.join("two.png"), image::ImageFormat::Png)
            .unwrap();
        assert_ne!(
            fs::metadata(source.join("one.jpg")).unwrap().len(),
            fs::metadata(source.join("two.png")).unwrap().len()
        );
        let summary = analyze(&cfg, source.to_str().unwrap(), "Unique").unwrap();
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*)FROM job_items WHERE job_id=?1 AND sha256 IS NULL",
                [&summary.job_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
        assert_eq!(
            conn.query_row(
                "SELECT bytes FROM job_metrics WHERE job_id=?1 AND stage='hashing_workers'",
                [&summary.job_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            0
        );
        drop(conn);
        consolidate(&cfg, &summary.job_id).unwrap();
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*)FROM assets WHERE length(hash)=64",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn filename_collision_never_overwrites_distinct_content() {
        let (root, cfg) = fixture();
        let first_source = root.join("cartao-a");
        let second_source = root.join("cartao-b");
        fs::create_dir_all(&first_source).unwrap();
        fs::create_dir_all(&second_source).unwrap();
        test_image(&first_source.join("IMG_0001.jpg"), [1, 2, 3]);
        test_image(&second_source.join("IMG_0001.jpg"), [4, 5, 6]);
        let first = analyze(&cfg, first_source.to_str().unwrap(), "Cartao A").unwrap();
        consolidate(&cfg, &first.job_id).unwrap();
        let second = analyze(&cfg, second_source.to_str().unwrap(), "Cartao B").unwrap();
        consolidate(&cfg, &second.job_id).unwrap();
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        let mut stmt = conn
            .prepare("SELECT master_path FROM assets ORDER BY master_path")
            .unwrap();
        let paths: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(paths.len(), 2);
        assert_ne!(paths[0], paths[1]);
        assert!(paths.iter().any(|p| p.contains("IMG_0001-")));
        drop(stmt);
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inaccessible_and_recursive_sources_are_rejected() {
        let (root, cfg) = fixture();
        let missing = root.join("nao-existe");
        assert!(analyze(&cfg, missing.to_str().unwrap(), "Ausente").is_err());
        assert!(analyze(&cfg, root.to_str().unwrap(), "Recursiva").is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupted_media_is_quarantined_and_source_is_untouched() {
        let (root, cfg) = fixture();
        let source = root.join("camera");
        fs::create_dir_all(&source).unwrap();
        let path = source.join("broken.jpg");
        let original = b"not a jpeg";
        fs::write(&path, original).unwrap();
        let summary = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        assert_eq!(summary.invalid, 1);
        assert_eq!(summary.new_files, 0);
        assert_eq!(fs::read(&path).unwrap(), original);
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row("SELECT state FROM media_validation", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            "corrupted"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap()
    }
    #[test]
    fn interrupted_analysis_resumes_without_duplicate_items() {
        let (root, cfg) = fixture();
        let source = root.join("camera");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("one.jpg"), [1, 2, 3]);
        let first = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        let second = analyze_with_job(
            &cfg,
            source.to_str().unwrap(),
            "Camera",
            Some(&first.job_id),
        )
        .unwrap();
        assert_eq!(second.new_files, 1);
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM job_items WHERE job_id=?1",
                [first.job_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap()
    }

    #[test]
    fn job_controls_are_persistent_and_cancel_is_observed() {
        let (root, cfg) = fixture();
        let source = root.join("camera");
        fs::create_dir_all(&source).unwrap();
        test_image(&source.join("IMG_0001.jpg"), [1, 2, 3]);
        let summary = analyze(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        assert_eq!(
            set_job_state(&cfg, &summary.job_id, "paused")
                .unwrap()
                .state,
            "pausing"
        );
        assert_eq!(
            set_job_state(&cfg, &summary.job_id, "running")
                .unwrap()
                .state,
            "consolidating"
        );
        assert_eq!(
            set_job_state(&cfg, &summary.job_id, "canceled")
                .unwrap()
                .state,
            "canceling"
        );
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            control_point(&conn, &summary.job_id).unwrap_err(),
            "JOB_CANCELED"
        );
        assert_eq!(
            job_progress(&cfg, &summary.job_id).unwrap().state,
            "canceled"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canceled_analysis_never_modifies_source() {
        let (root, cfg) = fixture();
        let source = root.join("camera");
        fs::create_dir_all(&source).unwrap();
        let original = source.join("IMG_0001.jpg");
        test_image(&original, [8, 9, 10]);
        let before = fs::read(&original).unwrap();
        let job = queue_analysis(&cfg, source.to_str().unwrap(), "Camera").unwrap();
        let token = crate::process::CancellationToken::default();
        token.cancel();
        assert_eq!(
            analyze_with_job_cancel(&cfg, source.to_str().unwrap(), "Camera", Some(&job), &token)
                .unwrap_err(),
            "JOB_CANCELED"
        );
        assert_eq!(fs::read(&original).unwrap(), before);
        assert!(!source.join("IMG_0001.jpg.lumina-part").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn analyzes_two_thousand_small_files_without_losing_history() {
        let (root, cfg) = fixture();
        let source = root.join("batch");
        fs::create_dir_all(&source).unwrap();
        for index in 0..2_000 {
            fs::write(
                source.join(format!("IMG_{index:05}.jpg")),
                b"invalid-but-inventoried",
            )
            .unwrap();
        }
        let summary = analyze(&cfg, source.to_str().unwrap(), "Lote").unwrap();
        assert_eq!((summary.discovered, summary.invalid), (2_000, 2_000));
        let conn =
            catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM job_items WHERE job_id=?1",
                [summary.job_id],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            2_000
        );
        assert_eq!(
            fs::read(source.join("IMG_00000.jpg")).unwrap(),
            b"invalid-but-inventoried"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[cfg(windows)]
    fn imports_a_large_video_without_changing_the_source() {
        let (root, cfg) = fixture();
        let source = root.join("video");
        fs::create_dir_all(&source).unwrap();
        let video = source.join("large.mp4");
        crate::process::run(
            crate::process::ProcessSpec::new("FFmpeg", "ffmpeg").args([
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=320x180:d=2",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                video.to_string_lossy().as_ref(),
            ]),
            &crate::process::CancellationToken::default(),
        )
        .unwrap();
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(&video)
            .unwrap();
        file.set_len(64 * 1024 * 1024).unwrap();
        drop(file);
        let before = crate::storage::sha256(&video).unwrap();
        let summary = analyze(&cfg, source.to_str().unwrap(), "Vídeo grande").unwrap();
        assert_eq!(summary.new_files, 1);
        consolidate(&cfg, &summary.job_id).unwrap();
        assert_eq!(crate::storage::sha256(&video).unwrap(), before);
        assert_eq!(verify(&cfg).unwrap().errors, 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn metadata_batches_publish_incremental_progress_and_observe_cancel() {
        let paths = (0..51)
            .map(|index| PathBuf::from(format!("missing-{index}.jpg")))
            .collect::<Vec<_>>();
        let token = crate::process::CancellationToken::default();
        let mut updates = Vec::new();
        let _ = capture_metadata_batches(&paths, &token, |done, total, _| {
            updates.push((done, total));
        });
        assert_eq!(updates.first(), Some(&(0, 1)));
        assert_eq!(updates.last(), Some(&(1, 1)));

        token.cancel();
        let mut canceled_updates = 0;
        let _ = capture_metadata_batches(&paths, &token, |_, _, _| canceled_updates += 1);
        assert_eq!(canceled_updates, 0);
    }

    #[test]
    #[cfg(windows)]
    fn raw_validation_is_reused_from_the_metadata_batch() {
        let raw = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample.dng");
        let (_, validations) = capture_metadata_batches(
            std::slice::from_ref(&raw),
            &crate::process::CancellationToken::default(),
            |_, _, _| {},
        );
        let result = validations.get(&cache_key(&raw)).unwrap();
        assert!(result.state.accepted(), "{}", result.details);
        assert_eq!(result.tool, "exiftool-batch");
    }
    #[test]
    fn manifest_is_versioned_complete_and_checksummed() {
        use sha2::{Digest, Sha256};
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(&master).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'file',1,?3,?2)",params!["a".repeat(64),now,master.join("a.jpg").to_string_lossy()]).unwrap();
        conn.execute("INSERT INTO backup_entries(asset_id,path,hash,verified_at,state)VALUES('a',?1,?2,?3,'verified')",params![backup.join("a.jpg").to_string_lossy(),"a".repeat(64),now]).unwrap();
        write_backup_manifest(&conn, &cfg).unwrap();
        let contents = fs::read_to_string(backup.join("manifest.jsonl")).unwrap();
        let mut lines = contents.lines();
        let header: serde_json::Value = serde_json::from_str(lines.next().unwrap()).unwrap();
        let payload = lines.collect::<Vec<_>>().join("\n");
        assert_eq!(header["schemaVersion"], 1);
        assert_eq!(header["entries"], 1);
        assert_eq!(
            header["payloadSha256"],
            format!("{:x}", Sha256::digest(payload.as_bytes()))
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn gallery_queries_remain_fast_while_thumbnails_are_generated() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(&master).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        for index in 0..40 {
            let path = master.join(format!("{index}.png"));
            image::RgbImage::from_pixel(320, 240, image::Rgb([index as u8, 2, 3]))
                .save(&path)
                .unwrap();
            let hash = crate::storage::sha256(&path).unwrap();
            conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES(?1,?2,?3,'photo','png',?4,'file',1,?5,?4)",params![format!("a-{index}"),hash,format!("{index}.png"),Utc::now().to_rfc3339(),path.to_string_lossy()]).unwrap();
            crate::media::enqueue_thumbnail(&cfg, &format!("a-{index}"), 1).unwrap();
        }
        drop(conn);
        let worker_cfg = cfg.clone();
        let worker = std::thread::spawn(move || {
            process_thumbnail_queue(
                &worker_cfg,
                "_thumbnail_background",
                &crate::process::CancellationToken::default(),
            )
            .unwrap()
        });
        let mut latencies = Vec::new();
        for _ in 0..20 {
            let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
            let started = Instant::now();
            let result = crate::gallery::search(
                &conn,
                &GalleryRequest {
                    filters: GalleryFilters::default(),
                    cursor: None,
                    limit: Some(40),
                },
            )
            .unwrap();
            assert_eq!(result.assets.len(), 40);
            latencies.push(started.elapsed().as_millis());
        }
        worker.join().unwrap();
        latencies.sort_unstable();
        let p95 = latencies[latencies.len() * 95 / 100];
        assert!(p95 < 300, "p95 da galeria sob carga: {p95} ms");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn technical_inventory_is_persistent_idempotent_and_detects_content_mismatch() {
        let root = std::env::temp_dir().join(format!("lumina-formats-{}", Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(&master).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: master.to_string_lossy().into(),
            backup_path: backup.to_string_lossy().into(),
            created_at: Utc::now().to_rfc3339(),
        };
        let path = master.join("wrong.png");
        fs::write(&path, [0xff, 0xd8, 0xff, 0xd9]).unwrap();
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'wrong.png','photo','png',?2,'file',4,?3,?2)",params!["a".repeat(64),Utc::now().to_rfc3339(),path.to_string_lossy()]).unwrap();
        drop(conn);
        let job = queue_format_enrichment(&cfg).unwrap();
        enrich_formats_job(&cfg, &job, &crate::process::CancellationToken::default()).unwrap();
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        let value:(String,bool)=conn.query_row("SELECT detected_format,extension_matches FROM asset_technical_metadata WHERE asset_id='a'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(value, ("jpeg".into(), false));
        assert_eq!(
            conn.query_row("SELECT COUNT(*)FROM asset_technical_metadata", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
