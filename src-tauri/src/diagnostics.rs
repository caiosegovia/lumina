use chrono::Utc;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    panic,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static FRONTEND_HEARTBEAT_MS: AtomicU64 = AtomicU64::new(0);

pub struct ActiveOperation {
    marker: PathBuf,
}

impl Drop for ActiveOperation {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.marker);
    }
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

pub fn frontend_heartbeat() {
    FRONTEND_HEARTBEAT_MS.store(epoch_ms(), Ordering::Relaxed);
}

fn root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Lumina/diagnostics")
}

pub fn begin_operation(kind: &str, detail: &str) -> ActiveOperation {
    let directory = root();
    let _ = fs::create_dir_all(&directory);
    let marker = directory.join("active.operation");
    let safe_detail = detail
        .replace(['\r', '\n'], " ")
        .chars()
        .take(500)
        .collect::<String>();
    let record = format!("{}\t{}\t{}", Utc::now().to_rfc3339(), kind, safe_detail);
    let _ = fs::write(&marker, record);
    ActiveOperation { marker }
}

pub fn active_operation() -> Option<String> {
    fs::read_to_string(root().join("active.operation")).ok()
}

pub fn append(kind: &str, detail: &str) {
    let directory = root();
    if fs::create_dir_all(&directory).is_err() {
        return;
    }
    let log = directory.join("session.log");
    if fs::metadata(&log)
        .map(|meta| meta.len() > 512 * 1024)
        .unwrap_or(false)
    {
        let previous = directory.join("session.previous.log");
        let _ = fs::remove_file(&previous);
        let _ = fs::rename(&log, previous);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log) {
        let safe_detail = detail
            .replace(['\r', '\n'], " ")
            .chars()
            .take(500)
            .collect::<String>();
        let _ = writeln!(
            file,
            "{}\t{}\t{}",
            Utc::now().to_rfc3339(),
            kind,
            safe_detail
        );
    }
}

pub fn runtime_sample(
    active_job: Option<&str>,
    stage: Option<&str>,
    queue: Option<(i64, i64, i64)>,
) {
    let (pending, processing, failed) = queue.unwrap_or_default();
    let memory = working_set_bytes();
    let heartbeat = FRONTEND_HEARTBEAT_MS.load(Ordering::Relaxed);
    let heartbeat_age = if heartbeat == 0 {
        0
    } else {
        epoch_ms().saturating_sub(heartbeat)
    };
    append(
        "runtime_metric",
        &format!(
            "pid={} working_set_bytes={} process_cpu_ms={} ui_heartbeat_age_ms={} active_job={} stage={} queue_pending={} queue_processing={} queue_failed={}",
            std::process::id(),
            memory,
            process_cpu_ms(),
            heartbeat_age,
            active_job.map(short_id).unwrap_or("none"),
            stage.unwrap_or("idle"),
            pending,
            processing,
            failed
        ),
    );
    if heartbeat_age > 15_000 || memory > 1_500_000_000 {
        append(
            "runtime_warning",
            &format!(
                "working_set_bytes={memory} ui_heartbeat_age_ms={heartbeat_age} stage={}",
                stage.unwrap_or("idle")
            ),
        );
    }
}

fn short_id(value: &str) -> &str {
    value.get(..value.len().min(12)).unwrap_or(value)
}

#[cfg(windows)]
fn working_set_bytes() -> u64 {
    use windows_sys::Win32::{
        System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        System::Threading::GetCurrentProcess,
    };
    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        PageFaultCount: 0,
        PeakWorkingSetSize: 0,
        WorkingSetSize: 0,
        QuotaPeakPagedPoolUsage: 0,
        QuotaPagedPoolUsage: 0,
        QuotaPeakNonPagedPoolUsage: 0,
        QuotaNonPagedPoolUsage: 0,
        PagefileUsage: 0,
        PeakPagefileUsage: 0,
    };
    let ok = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    if ok == 0 {
        0
    } else {
        counters.WorkingSetSize as u64
    }
}

#[cfg(not(windows))]
fn working_set_bytes() -> u64 {
    0
}

#[cfg(windows)]
fn process_cpu_ms() -> u64 {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentProcess, GetProcessTimes},
    };
    let mut created: FILETIME = unsafe { std::mem::zeroed() };
    let mut exited: FILETIME = unsafe { std::mem::zeroed() };
    let mut kernel: FILETIME = unsafe { std::mem::zeroed() };
    let mut user: FILETIME = unsafe { std::mem::zeroed() };
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut created,
            &mut exited,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return 0;
    }
    let ticks =
        |value: FILETIME| ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64;
    (ticks(kernel) + ticks(user)) / 10_000
}
#[cfg(not(windows))]
fn process_cpu_ms() -> u64 {
    0
}

pub fn log_files() -> Vec<PathBuf> {
    ["session.log", "session.previous.log"]
        .into_iter()
        .map(|name| root().join(name))
        .filter(|path| path.is_file())
        .collect()
}

pub fn spawn_monitor(cfg: crate::models::LibraryConfig) {
    let _ = std::thread::Builder::new()
        .name("lumina-observability".into())
        .spawn(move || loop {
            let snapshot = crate::catalog::open(
                &std::path::Path::new(&cfg.master_path).join(".lumina/catalog.sqlite"),
            )
            .ok()
            .map(|conn| {
                let active = conn
                    .query_row(
                        "SELECT id,stage FROM jobs WHERE state IN('queued','analyzing','consolidating','protecting','pausing','canceling') ORDER BY updated_at DESC LIMIT 1",
                        [],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                    .ok();
                let queue = conn
                    .query_row(
                        "SELECT COALESCE(SUM(state='pending'),0),COALESCE(SUM(state='processing'),0),COALESCE(SUM(state='failed'),0) FROM work_queue",
                        [],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                    )
                    .ok();
                (active, queue)
            });
            match snapshot {
                Some((active, queue)) => runtime_sample(
                    active.as_ref().map(|item| item.0.as_str()),
                    active.as_ref().map(|item| item.1.as_str()),
                    queue,
                ),
                None => runtime_sample(None, Some("catalog_unavailable"), None),
            }
            std::thread::sleep(std::time::Duration::from_secs(15));
        });
}

pub fn start_session() {
    let directory = root();
    let marker = directory.join("running.session");
    if marker.exists() {
        append(
            "previous_session_abnormal",
            "O aplicativo não registrou encerramento normal",
        );
    }
    if let Some(operation) = active_operation() {
        append("previous_operation_incomplete", &operation);
        let _ = fs::remove_file(root().join("active.operation"));
    }
    let _ = fs::create_dir_all(&directory);
    let _ = fs::write(&marker, Utc::now().to_rfc3339());
    append("session_started", env!("CARGO_PKG_VERSION"));
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let location = info
            .location()
            .and_then(|location| std::path::Path::new(location.file()).file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        append(
            "panic",
            &format!(
                "componente={location} linha={}",
                info.location().map(|value| value.line()).unwrap_or(0)
            ),
        );
        default_hook(info);
    }));
}

pub fn finish_session() {
    append("session_finished", "clean");
    let _ = fs::remove_file(root().join("running.session"));
}

pub fn client_error(kind: &str, detail: &str) {
    let safe_kind = match kind {
        "frontend_error" | "unhandled_rejection" | "media_error" => kind,
        _ => "client_error",
    };
    append(safe_kind, &crate::process::sanitize(detail));
}
