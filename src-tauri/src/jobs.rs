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
    sync::{Arc, Mutex},
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
    pending_analysis: Mutex<VecDeque<PendingAnalysis>>,
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
                pending_analysis: Mutex::new(VecDeque::new()),
            }),
        }
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
        let next = self
            .inner
            .pending_analysis
            .lock()
            .ok()
            .and_then(|mut q| q.pop_front());
        if let Some(next) = next {
            if self.reserve(&next.job).is_ok() {
                if let Err(error) = self.spawn_analysis(next.clone()) {
                    self.mark_failed(&next.cfg, &next.job, &error);
                    self.release(&next.job);
                }
            } else if let Ok(mut queue) = self.inner.pending_analysis.lock() {
                queue.push_front(next);
            }
        }
    }
    fn mark_failed(&self, cfg: &LibraryConfig, job: &str, error: &str) {
        if let Ok(conn) = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
        {
            let now = Utc::now().to_rfc3339();
            let _ = conn.execute("UPDATE jobs SET state='failed',interruption_reason=?2,finished_at=?3,updated_at=?3 WHERE id=?1", params![job,error,now]);
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
        if let Ok(mut pending) = self.inner.pending_analysis.lock() {
            pending.retain(|item| item.job != job);
        }
    }
    pub fn start_analysis(
        &self,
        cfg: LibraryConfig,
        path: String,
        name: String,
    ) -> Result<String, String> {
        let job = engine::queue_analysis(&cfg, &path, &name)?;
        let pending = PendingAnalysis {
            cfg,
            path,
            name,
            job: job.clone(),
        };
        match self.reserve(&job) {
            Ok(()) => self.spawn_analysis(pending)?,
            Err(_) => self
                .inner
                .pending_analysis
                .lock()
                .map_err(|_| "Fila de análises indisponível")?
                .push_back(pending),
        }
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
        if matches!(
            stage.as_str(),
            "ready" | "batch_pending" | "copying" | "thumbnail" | "completed"
        ) {
            return self.start_consolidation(cfg, job);
        }
        if matches!(
            stage.as_str(),
            "protection_pending" | "backup" | "waiting_backup_space" | "backup_error"
        ) {
            return self.start_protection(cfg, job);
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
        conn.execute("UPDATE jobs SET state='interrupted',interruption_reason='Aplicativo encerrado durante o processamento',updated_at=?1 WHERE state IN('queued','analyzing','consolidating','protecting','pausing','paused','canceling')",[now]).map_err(|e|e.to_string())?;
        Ok(())
    }
    pub fn recoverable(cfg: &LibraryConfig) -> Result<Vec<RecoverableJob>, String> {
        let conn = catalog::open(&Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"))
            .map_err(|e| e.to_string())?;
        let mut stmt=conn.prepare("SELECT id,source_path,state,stage,interruption_reason,updated_at FROM jobs WHERE state='interrupted' ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
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
        .spawn(move || loop {
            match engine::job_progress(&cfg, &job) {
                Ok(snapshot) => {
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
                }
                Err(_) => break,
            }
            std::thread::sleep(std::time::Duration::from_millis(150));
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
    fn cancel_removes_an_analysis_that_is_still_queued() {
        let manager = JobManager::new();
        let cfg = LibraryConfig {
            id: "l".into(),
            name: "t".into(),
            master_path: "master".into(),
            backup_path: "backup".into(),
            created_at: Utc::now().to_rfc3339(),
        };
        manager
            .inner
            .pending_analysis
            .lock()
            .unwrap()
            .push_back(PendingAnalysis {
                cfg,
                path: "source".into(),
                name: "source".into(),
                job: "queued".into(),
            });
        manager.cancel("queued");
        assert!(manager.inner.pending_analysis.lock().unwrap().is_empty());
    }
}
