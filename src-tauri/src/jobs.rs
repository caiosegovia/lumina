use crate::{
    catalog, engine,
    models::{LibraryConfig, RecoverableJob},
    process::CancellationToken,
};
use chrono::Utc;
use rusqlite::params;
use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::Emitter;
use uuid::Uuid;

#[derive(Clone)]
pub struct JobManager {
    inner: Arc<Inner>,
}
struct Inner {
    instance_id: String,
    active: Mutex<Option<String>>,
    tokens: Mutex<HashMap<String, CancellationToken>>,
    library: Mutex<Option<LibraryConfig>>,
    thumbnail_worker_active: AtomicBool,
    thumbnail_dispatcher_active: AtomicBool,
    thumbnail_requests: Mutex<VecDeque<(LibraryConfig, String)>>,
}
#[derive(Clone)]
struct PendingAnalysis {
    cfg: LibraryConfig,
    path: String,
    name: String,
    job: String,
}
impl JobManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                instance_id: Uuid::new_v4().to_string(),
                active: Mutex::new(None),
                tokens: Mutex::new(HashMap::new()),
                library: Mutex::new(None),
                thumbnail_worker_active: AtomicBool::new(false),
                thumbnail_dispatcher_active: AtomicBool::new(false),
                thumbnail_requests: Mutex::new(VecDeque::new()),
            }),
        }
    }
    pub fn request_thumbnail(&self, cfg: LibraryConfig, asset: String) -> Result<(), String> {
        self.inner
            .thumbnail_requests
            .lock()
            .map_err(|_| "Fila de miniaturas indisponível".to_string())?
            .push_back((cfg, asset));
        if self
            .inner
            .thumbnail_dispatcher_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        let manager = self.clone();
        std::thread::Builder::new()
            .name("lumina-thumbnail-dispatcher".into())
            .spawn(move || loop {
                let request = manager
                    .inner
                    .thumbnail_requests
                    .lock()
                    .ok()
                    .and_then(|mut queue| queue.pop_front());
                let Some((cfg, asset)) = request else {
                    manager
                        .inner
                        .thumbnail_dispatcher_active
                        .store(false, Ordering::Release);
                    let pending = manager
                        .inner
                        .thumbnail_requests
                        .lock()
                        .map(|queue| !queue.is_empty())
                        .unwrap_or(false);
                    if pending
                        && manager
                            .inner
                            .thumbnail_dispatcher_active
                            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                    {
                        continue;
                    }
                    break;
                };
                if crate::media::enqueue_thumbnail(&cfg, &asset, 100).is_ok() {
                    let _ = manager.start_thumbnail_worker(cfg);
                }
            })
            .map_err(|error| {
                self.inner
                    .thumbnail_dispatcher_active
                    .store(false, Ordering::Release);
                error.to_string()
            })?;
        Ok(())
    }
    fn start_thumbnail_worker(&self, cfg: LibraryConfig) -> Result<(), String> {
        if self
            .inner
            .thumbnail_worker_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        let manager = self.clone();
        std::thread::Builder::new()
            .name("lumina-thumbnail-worker".into())
            .spawn(move || {
                let token = CancellationToken::default();
                loop {
                    let _ = engine::process_thumbnail_queue(&cfg, "_thumbnail_background", &token);
                    manager.inner.thumbnail_worker_active.store(false, Ordering::Release);
                    let pending=catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")).ok().and_then(|conn|conn.query_row("SELECT EXISTS(SELECT 1 FROM work_queue WHERE kind='thumbnail' AND state='pending')",[],|row|row.get::<_,bool>(0)).ok()).unwrap_or(false);
                    if pending&&manager.inner.thumbnail_worker_active.compare_exchange(false,true,Ordering::AcqRel,Ordering::Acquire).is_ok(){continue}
                    break
                }
            })
            .map_err(|error| {
                self.inner
                    .thumbnail_worker_active
                    .store(false, Ordering::Release);
                error.to_string()
            })?;
        Ok(())
    }
    pub fn resume_background(&self, cfg: LibraryConfig) -> Result<(), String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE work_queue SET state='pending',updated_at=?1 WHERE kind='thumbnail' AND state='processing'",
            [&now],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE work_queue SET state='completed',last_error=NULL,updated_at=?1 WHERE kind='thumbnail' AND EXISTS(SELECT 1 FROM thumbnails t WHERE t.asset_id=work_queue.asset_id AND t.state='ready' AND t.generator_version=?2)",
            params![&now, crate::media::THUMBNAIL_VERSION],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM work_queue WHERE kind='thumbnail' AND id NOT IN(SELECT MIN(id) FROM work_queue WHERE kind='thumbnail' GROUP BY asset_id)",
            [],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE work_queue SET state='pending',updated_at=?1 WHERE kind='thumbnail' AND NOT EXISTS(SELECT 1 FROM thumbnails t WHERE t.asset_id=work_queue.asset_id AND t.state='ready' AND t.generator_version=?2)",
            params![&now, crate::media::THUMBNAIL_VERSION],
        )
        .map_err(|e| e.to_string())?;
        let pending:bool=conn.query_row("SELECT EXISTS(SELECT 1 FROM work_queue WHERE kind='thumbnail' AND state='pending')",[],|row|row.get(0)).map_err(|e|e.to_string())?;
        drop(conn);
        if pending {
            self.start_thumbnail_worker(cfg)?
        }
        Ok(())
    }
    fn reserve(&self, job: &str) -> Result<(), String> {
        let mut active = self
            .inner
            .active
            .lock()
            .map_err(|_| "Gerenciador indisponível")?;
        if let Some(current) = active.as_ref() {
            return Err(format!(
                "O trabalho {current} já está escrevendo nesta biblioteca"
            ));
        }
        *active = Some(job.into());
        self.inner
            .tokens
            .lock()
            .unwrap()
            .insert(job.into(), CancellationToken::default());
        Ok(())
    }
    fn release(&self, job: &str) {
        {
            let mut active = self.inner.active.lock().unwrap();
            if active.as_deref() == Some(job) {
                *active = None
            }
        }
        self.inner.tokens.lock().unwrap().remove(job);
        let cfg = self.inner.library.lock().ok().and_then(|cfg| cfg.clone());
        let next = cfg.and_then(|cfg| {
            let path = Path::new(&cfg.master_path).join(".lumina/catalog.sqlite");
            catalog::open(&path).ok().and_then(|conn| {
                conn.query_row(
                    "SELECT id,source_path,COALESCE((SELECT name FROM sources WHERE id=jobs.source_id),'Fonte') FROM jobs WHERE state='queued' AND stage='discovery' ORDER BY created_at,id LIMIT 1",
                    [],
                    |row| Ok(PendingAnalysis { cfg: cfg.clone(), job: row.get(0)?, path: row.get(1)?, name: row.get(2)? }),
                ).ok()
            })
        });
        if let Some(next) = next {
            if self.reserve(&next.job).is_ok() {
                if let Err(error) = self.spawn_analysis(next.clone()) {
                    self.mark_failed(&next.cfg, &next.job, &error);
                    self.release(&next.job);
                }
            }
        }
    }
    fn mark_failed(&self, cfg: &LibraryConfig, job: &str, error: &str) {
        if let Ok(conn) = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        {
            let now = Utc::now().to_rfc3339();
            let _ = conn.execute("UPDATE jobs SET state=CASE WHEN state IN('backup_error','waiting_backup_space') THEN state ELSE 'failed' END,interruption_reason=?2,finished_at=?3,updated_at=?3 WHERE id=?1", params![job,error,now]);
        }
    }
    fn mark_canceled(&self, cfg: &LibraryConfig, job: &str) {
        if let Ok(conn) = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        {
            let now = Utc::now().to_rfc3339();
            let stage: String = conn
                .query_row("SELECT stage FROM jobs WHERE id=?1", [job], |r| r.get(0))
                .unwrap_or_default();
            if matches!(stage.as_str(), "backup" | "backup_space_check") {
                let _=conn.execute("UPDATE work_queue SET state='pending',updated_at=?2 WHERE job_id=?1 AND kind='backup' AND state='processing'",params![job,now]);
                let _=conn.execute("UPDATE jobs SET state='protection_pending',stage='protection_pending',interruption_reason='Proteção interrompida pelo usuário',current_file=NULL,updated_at=?2 WHERE id=?1",params![job,now]);
                return;
            }
            if stage == "technical_enrichment" {
                let _=conn.execute("UPDATE work_queue SET state='pending',updated_at=?2 WHERE job_id=?1 AND kind='technical_metadata' AND state='processing'",params![job,now]);
            }
            let _ = conn.execute("UPDATE job_items SET state='queued',current_stage='validation',updated_at=?2 WHERE job_id=?1 AND state='processing'", params![job,now]);
            let _ = conn.execute("UPDATE jobs SET state='canceled',interruption_reason='Cancelado pelo usuário',current_file=NULL,finished_at=?2,updated_at=?2 WHERE id=?1", params![job,now]);
            let _ = conn.execute("INSERT INTO events(job_id,at,path,state,details)VALUES(?1,?2,'','canceled','Cancelamento concluído com segurança')", params![job,now]);
        }
    }
    fn spawn_analysis(&self, pending: PendingAnalysis) -> Result<(), String> {
        let cancel = self.token(&pending.job)?;
        let manager = self.clone();
        std::thread::Builder::new()
            .name(format!("lumina-analysis-{}", pending.job))
            .spawn(move || {
                let result = engine::analyze_with_job_cancel(
                    &pending.cfg,
                    &pending.path,
                    &pending.name,
                    Some(&pending.job),
                    &cancel,
                );
                if let Err(error) = result {
                    if error == "JOB_CANCELED" {
                        manager.mark_canceled(&pending.cfg, &pending.job);
                    } else {
                        manager.mark_failed(&pending.cfg, &pending.job, &error);
                    }
                }
                manager.release(&pending.job);
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn token(&self, job: &str) -> Result<CancellationToken, String> {
        self.inner
            .tokens
            .lock()
            .map_err(|_| "Gerenciador indisponível".to_string())?
            .get(job)
            .cloned()
            .ok_or_else(|| "Token de cancelamento do trabalho não encontrado".to_string())
    }
    pub fn cancel(&self, job: &str) {
        if let Ok(tokens) = self.inner.tokens.lock() {
            if let Some(token) = tokens.get(job) {
                token.cancel();
            }
        }
    }
    pub fn start_analysis(
        &self,
        cfg: LibraryConfig,
        path: String,
        name: String,
    ) -> Result<String, String> {
        *self
            .inner
            .library
            .lock()
            .map_err(|_| "Biblioteca indisponível")? = Some(cfg.clone());
        let job = engine::queue_analysis(&cfg, &path, &name)?;
        let pending = PendingAnalysis {
            cfg,
            path,
            name,
            job: job.clone(),
        };
        if self.reserve(&job).is_ok() {
            self.spawn_analysis(pending)?;
        }
        // O catálogo mantém o job em queued até o slot ficar livre.
        Ok(job)
    }
    pub fn start_source_sync(
        &self,
        cfg: LibraryConfig,
        source_id: String,
    ) -> Result<String, String> {
        if self.has_active() {
            return Err("Aguarde o trabalho atual terminar antes de sincronizar".into());
        }
        let job = crate::sync::queue(&cfg, &source_id)?;
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker = job.clone();
        std::thread::Builder::new()
            .name(format!("lumina-source-sync-{job}"))
            .spawn(move || {
                if let Err(error) = crate::sync::run(&cfg, &worker, &cancel) {
                    if error == "JOB_CANCELED" {
                        manager.mark_canceled(&cfg, &worker);
                    } else {
                        if let Ok(conn) = catalog::open(
                            &Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"),
                        ) {
                            let _ = conn.execute("UPDATE source_sync_settings SET last_state='failed',last_error=?2 WHERE source_id=(SELECT source_id FROM jobs WHERE id=?1)",params![worker,error]);
                        }
                        manager.mark_failed(&cfg, &worker, &error);
                    }
                }
                manager.release(&worker);
            })
            .map_err(|error| error.to_string())?;
        Ok(job)
    }
    pub fn start_consolidation(&self, cfg: LibraryConfig, job: String) -> Result<(), String> {
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker_job = job.clone();
        std::thread::Builder::new().name(format!("lumina-consolidation-{job}")).spawn(move||{if let Err(error)=engine::consolidate_cancel(&cfg,&worker_job,&cancel){if error=="JOB_CANCELED"{manager.mark_canceled(&cfg,&worker_job)}else if let Ok(conn)=catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")){let _=conn.execute("UPDATE jobs SET state='failed',interruption_reason=?2,updated_at=?3 WHERE id=?1",params![worker_job,error,Utc::now().to_rfc3339()]);}}else{let _=engine::process_thumbnail_queue(&cfg,&worker_job,&cancel);}manager.release(&worker_job)}).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn start_protection(&self, cfg: LibraryConfig, job: String) -> Result<(), String> {
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker = job.clone();
        std::thread::Builder::new()
            .name(format!("lumina-protection-{job}"))
            .spawn(move || {
                if let Err(error) = engine::protect_job(&cfg, &worker, &cancel) {
                    if error == "JOB_CANCELED" {
                        manager.mark_canceled(&cfg, &worker)
                    } else {
                        manager.mark_failed(&cfg, &worker, &error)
                    }
                }
                manager.release(&worker)
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    fn spawn_verification(&self, cfg: LibraryConfig, job: String) -> Result<(), String> {
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker = job.clone();
        std::thread::Builder::new()
            .name(format!("lumina-verification-{job}"))
            .spawn(move || {
                if let Err(error) = engine::verify_job(&cfg, &worker, &cancel) {
                    if error == "JOB_CANCELED" {
                        manager.mark_canceled(&cfg, &worker)
                    } else {
                        manager.mark_failed(&cfg, &worker, &error)
                    }
                }
                manager.release(&worker)
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn start_verification(&self, cfg: LibraryConfig) -> Result<String, String> {
        let job = engine::queue_verification(&cfg)?;
        self.spawn_verification(cfg, job.clone())?;
        Ok(job)
    }
    fn spawn_format_enrichment(&self, cfg: LibraryConfig, job: String) -> Result<(), String> {
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker = job.clone();
        std::thread::Builder::new()
            .name(format!("lumina-format-enrichment-{job}"))
            .spawn(move || {
                if let Err(error) = engine::enrich_formats_job(&cfg, &worker, &cancel) {
                    if error == "JOB_CANCELED" {
                        manager.mark_canceled(&cfg, &worker)
                    } else {
                        manager.mark_failed(&cfg, &worker, &error)
                    }
                }
                manager.release(&worker)
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn start_format_enrichment(&self, cfg: LibraryConfig) -> Result<String, String> {
        let job = engine::queue_format_enrichment(&cfg)?;
        self.spawn_format_enrichment(cfg, job.clone())?;
        Ok(job)
    }
    pub fn has_active(&self) -> bool {
        self.inner
            .active
            .lock()
            .map(|x| x.is_some())
            .unwrap_or(true)
    }
    pub fn resume(&self, cfg: LibraryConfig, job: String) -> Result<(), String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let (source,name,stage):(String,String,String)=conn.query_row("SELECT j.source_path,s.name,j.stage FROM jobs j JOIN sources s ON s.id=j.source_id WHERE j.id=?1",[&job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(|e|e.to_string())?;
        drop(conn);
        if stage == "thumbnail" {
            return self.resume_background(cfg);
        }
        if matches!(
            stage.as_str(),
            "ready" | "batch_pending" | "copying" | "completed"
        ) {
            return self.start_consolidation(cfg, job);
        }
        if matches!(
            stage.as_str(),
            "protection_pending" | "backup" | "waiting_backup_space" | "backup_error"
        ) {
            return self.start_protection(cfg, job);
        }
        if matches!(stage.as_str(), "verification" | "verification_error") {
            return self.spawn_verification(cfg, job);
        }
        if stage == "technical_enrichment" {
            return self.spawn_format_enrichment(cfg, job);
        }
        self.reserve(&job)?;
        let cancel = self.token(&job)?;
        let manager = self.clone();
        let worker_job = job.clone();
        std::thread::Builder::new().name(format!("lumina-resume-{job}")).spawn(move||{if let Err(error)=engine::analyze_with_job_cancel(&cfg,&source,&name,Some(&worker_job),&cancel){if error=="JOB_CANCELED"{manager.mark_canceled(&cfg,&worker_job)}else if let Ok(conn)=catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite")){let _=conn.execute("UPDATE jobs SET state='failed',interruption_reason=?2,updated_at=?3 WHERE id=?1",params![worker_job,error,Utc::now().to_rfc3339()]);}}manager.release(&worker_job)}).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn retry_failed(&self, cfg: &LibraryConfig, job: &str) -> Result<i64, String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let retry_stage:String=conn.query_row("SELECT CASE WHEN EXISTS(SELECT 1 FROM job_items WHERE job_id=?1 AND state IN('review','failed') AND COALESCE(last_error_kind,'validation') NOT IN('copy','backup')) THEN 'validation' ELSE 'copying' END",[job],|row|row.get(0)).map_err(|e|e.to_string())?;
        let changed=conn.execute("UPDATE job_items SET state='queued',attempts=attempts+1,updated_at=?2 WHERE job_id=?1 AND state IN('review','failed')",params![job,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        conn.execute("UPDATE jobs SET state='interrupted',stage=?2,interruption_reason='Itens preparados para nova tentativa',updated_at=?3 WHERE id=?1",params![job,retry_stage,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        Ok(changed as i64)
    }
    pub fn interrupt_running(cfg: &LibraryConfig) -> Result<(), String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE work_queue SET state='pending',updated_at=?1 WHERE state='processing'",
            [&now],
        )
        .map_err(|e| e.to_string())?;
        conn.execute("UPDATE jobs SET state='interrupted',interruption_reason='Aplicativo encerrado durante o processamento',updated_at=?1 WHERE state IN('queued','analyzing','consolidating','protecting','pausing','paused','canceling') AND source_path NOT LIKE 'lumina://%'",[now]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn recoverable(cfg: &LibraryConfig) -> Result<Vec<RecoverableJob>, String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let mut stmt=conn.prepare("SELECT id,source_path,state,stage,interruption_reason,updated_at FROM jobs WHERE state='interrupted' AND source_path NOT LIKE 'lumina://%' ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(RecoverableJob {
                    job_id: r.get(0)?,
                    source_path: r.get(1)?,
                    state: r.get(2)?,
                    stage: r.get(3)?,
                    interruption_reason: r.get(4)?,
                    updated_at: r.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }
    pub fn discard(cfg: &LibraryConfig, job: &str) -> Result<(), String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT temp_path FROM job_items WHERE job_id=?1 AND temp_path IS NOT NULL")
            .map_err(|e| e.to_string())?;
        let paths = stmt
            .query_map([job], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        drop(stmt);
        let owned = Path::new(&cfg.master_path).join(".lumina/temp").join(job);
        for value in paths {
            let path = Path::new(&value);
            if path.starts_with(&owned) && path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
        conn.execute("UPDATE jobs SET state='canceled',interruption_reason='Descartado pelo usuário',finished_at=?2,updated_at=?2 WHERE id=?1",params![job,Utc::now().to_rfc3339()]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn instance_id(&self) -> &str {
        &self.inner.instance_id
    }
}

pub fn emit_progress(app: tauri::AppHandle, cfg: LibraryConfig, job: String) {
    let _ = std::thread::Builder::new()
        .name(format!("lumina-events-{job}"))
        .spawn(move || {
            while let Ok(snapshot) = engine::job_progress(&cfg, &job) {
                let terminal = matches!(
                    snapshot.state.as_str(),
                    "ready"
                        | "batch_pending"
                        | "protection_pending"
                        | "waiting_space"
                        | "waiting_backup_space"
                        | "completed"
                        | "backup_error"
                        | "failed"
                        | "canceled"
                        | "interrupted"
                );
                let _ = app.emit("job-progress", &snapshot);
                if terminal {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discard_removes_only_job_owned_temporaries() {
        let root = std::env::temp_dir().join(Uuid::new_v4().to_string());
        let master = root.join("master");
        let backup = root.join("backup");
        let owned_dir = master.join(".lumina/temp/job");
        fs::create_dir_all(&owned_dir).unwrap();
        fs::create_dir_all(&backup).unwrap();
        let owned = owned_dir.join("1.part");
        let outside = root.join("outside.part");
        fs::write(&owned, b"owned").unwrap();
        fs::write(&outside, b"keep").unwrap();
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
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,created_at,updated_at)VALUES('job','s','source','interrupted',?1,?1)",[Utc::now().to_rfc3339()]).unwrap();
        for (id, path) in [(1, owned.to_string_lossy()), (2, outside.to_string_lossy())] {
            conn.execute("INSERT INTO job_items(id,job_id,source_path,filename,extension,media_type,temp_path,created_at,updated_at)VALUES(?1,'job',?2,'f','jpg','photo',?3,?4,?4)",params![id,format!("source-{id}"),path,Utc::now().to_rfc3339()]).unwrap();
        }
        drop(conn);
        JobManager::discard(&cfg, "job").unwrap();
        assert!(!owned.exists());
        assert!(outside.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn analysis_queue_configuration_is_kept_without_a_volatile_job_queue() {
        let manager = JobManager::new();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: "master".into(),
            backup_path: "backup".into(),
            created_at: Utc::now().to_rfc3339(),
        };
        *manager.inner.library.lock().unwrap() = Some(cfg);
        assert!(manager.inner.library.lock().unwrap().is_some());
    }

    #[test]
    fn restart_recovers_processing_work_from_the_catalog() {
        let root = std::env::temp_dir().join(format!("lumina-restart-{}", Uuid::new_v4()));
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
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sources(id,name,path,volume_label)VALUES('s','s','source','v')",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at)VALUES('j','s','source','analyzing','validation',?1,?1)",[&now]).unwrap();
        conn.execute("INSERT INTO work_queue(job_id,kind,state,created_at,updated_at)VALUES('j','verification','processing',?1,?1)",[&now]).unwrap();
        drop(conn);
        JobManager::interrupt_running(&cfg).unwrap();
        assert_eq!(JobManager::recoverable(&cfg).unwrap().len(), 1);
        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row("SELECT state FROM work_queue WHERE job_id='j'", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap(),
            "pending"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn thumbnail_maintenance_recovers_silently_and_deduplicates_legacy_work() {
        let root = std::env::temp_dir().join(format!("lumina-thumbs-restart-{}", Uuid::new_v4()));
        let master = root.join("master");
        let backup = root.join("backup");
        fs::create_dir_all(master.join(".lumina/cache/thumbnails")).unwrap();
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
        conn.execute("INSERT INTO sources(id,name,path,volume_label)VALUES('s','s','source','v'),('_lumina_maintenance','Manutenção','lumina://maintenance','internal')",[]).unwrap();
        conn.execute("INSERT INTO jobs(id,source_id,source_path,state,stage,created_at,updated_at)VALUES('import','s','source','queued','thumbnail',?1,?1),('_thumbnail_background','_lumina_maintenance','lumina://thumbnails','queued','thumbnail',?1,?1)",[&now]).unwrap();
        conn.execute("INSERT INTO assets(id,hash,filename,media_type,extension,captured_at,date_source,bytes,master_path,created_at)VALUES('a',?1,'a.jpg','photo','jpg',?2,'file',1,'a.jpg',?2)",params!["a".repeat(64),&now]).unwrap();
        conn.execute("INSERT INTO thumbnails(asset_id,generator_version,path,state,updated_at)VALUES('a',?1,'thumb.jpg','ready',?2)",params![crate::media::THUMBNAIL_VERSION,&now]).unwrap();
        conn.execute("INSERT INTO work_queue(job_id,asset_id,kind,state,created_at,updated_at)VALUES('import','a','thumbnail','completed',?1,?1),('_thumbnail_background','a','thumbnail','processing',?1,?1)",[&now]).unwrap();
        drop(conn);

        JobManager::interrupt_running(&cfg).unwrap();
        let recoverable = JobManager::recoverable(&cfg).unwrap();
        assert_eq!(recoverable.len(), 1);
        assert_eq!(recoverable[0].job_id, "import");
        JobManager::new().resume_background(cfg.clone()).unwrap();

        let conn = catalog::open(&master.join(".lumina/catalog.sqlite")).unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT state FROM jobs WHERE id='_thumbnail_background'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            "queued"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM work_queue WHERE asset_id='a' AND kind='thumbnail'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT state FROM work_queue WHERE asset_id='a' AND kind='thumbnail'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
            "completed"
        );
        drop(conn);
        fs::remove_dir_all(root).unwrap();
    }
}
