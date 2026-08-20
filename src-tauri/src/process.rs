use std::{
    ffi::{OsStr, OsString},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const PROCESS_LIMIT: usize = 2;
#[cfg(test)]
static TEST_PEAK_ACTIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst)
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
pub struct ProcessSpec {
    pub tool: String,
    pub logical_command: String,
    pub program: OsString,
    pub args: Vec<OsString>,
    pub timeout: Duration,
}
impl ProcessSpec {
    pub fn new(tool: impl Into<String>, program: impl Into<OsString>) -> Self {
        let tool = tool.into();
        Self {
            logical_command: tool.clone(),
            tool,
            program: program.into(),
            args: vec![],
            timeout: Duration::from_secs(30),
        }
    }
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|v| v.as_ref().to_os_string()));
        self
    }
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }
    pub fn logical(mut self, value: impl Into<String>) -> Self {
        self.logical_command = value.into();
        self
    }
}
#[derive(Debug)]
pub struct ProcessOutcome {
    pub tool: String,
    pub logical_command: String,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessErrorKind {
    MissingDependency,
    Timeout,
    Canceled,
    Spawn,
    Failed,
}
#[derive(Debug, Clone)]
pub struct ProcessError {
    pub kind: ProcessErrorKind,
    pub message: String,
}
struct Limiter {
    active: Mutex<usize>,
    available: Condvar,
}
struct Permit(&'static Limiter);
impl Drop for Permit {
    fn drop(&mut self) {
        let mut n = self.0.active.lock().unwrap();
        *n -= 1;
        self.0.available.notify_one()
    }
}
impl Limiter {
    fn acquire(&'static self) -> Permit {
        let mut n = self.active.lock().unwrap();
        while *n >= PROCESS_LIMIT {
            n = self.available.wait(n).unwrap()
        }
        *n += 1;
        #[cfg(test)]
        TEST_PEAK_ACTIVE.fetch_max(*n, Ordering::SeqCst);
        Permit(self)
    }
}
fn limiter() -> &'static Limiter {
    static VALUE: OnceLock<Limiter> = OnceLock::new();
    VALUE.get_or_init(|| Limiter {
        active: Mutex::new(0),
        available: Condvar::new(),
    })
}
fn resolve_program(program: &OsStr) -> OsString {
    let requested = Path::new(program);
    if requested.components().count() > 1 {
        return program.to_os_string();
    }
    let filename = if cfg!(windows) && requested.extension().is_none() {
        PathBuf::from(format!("{}.exe", requested.to_string_lossy()))
    } else {
        requested.to_path_buf()
    };
    let mut roots = Vec::new();
    if let Ok(executable) = std::env::current_exe() {
        if let Some(parent) = executable.parent() {
            roots.push(parent.join("tools"));
            roots.push(parent.join("resources/tools"));
        }
    }
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tools"));
    roots
        .into_iter()
        .map(|root| root.join(&filename))
        .find(|candidate| candidate.is_file())
        .map(|path| path.into_os_string())
        .unwrap_or_else(|| program.to_os_string())
}
pub fn sanitize(value: &str) -> String {
    value
        .split_whitespace()
        .map(|part| {
            let lower = part.to_ascii_lowercase();
            if lower.contains("token=")
                || lower.contains("password=")
                || lower.contains("secret=")
                || lower.contains("authorization:")
            {
                "[REDACTED]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(4000)
        .collect()
}
pub fn run(spec: ProcessSpec, cancel: &CancellationToken) -> Result<ProcessOutcome, ProcessError> {
    if cancel.is_cancelled() {
        return Err(ProcessError {
            kind: ProcessErrorKind::Canceled,
            message: "Operação cancelada".into(),
        });
    }
    let _permit = limiter().acquire();
    let started = Instant::now();
    let resolved_program = resolve_program(&spec.program);
    let mut command = Command::new(&resolved_program);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    let mut child = command.spawn().map_err(|error| {
        let missing = error.kind() == std::io::ErrorKind::NotFound;
        ProcessError {
            kind: if missing {
                ProcessErrorKind::MissingDependency
            } else {
                ProcessErrorKind::Spawn
            },
            message: if missing {
                format!(
                    "{} não foi encontrado. Instale a dependência e confirme que ela está no PATH.",
                    spec.tool
                )
            } else {
                format!("Não foi possível iniciar {}: {error}", spec.tool)
            },
        }
    })?;
    let mut stdout = child.stdout.take().expect("stdout configurado como pipe");
    let mut stderr = child.stderr.take().expect("stderr configurado como pipe");
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });
    let status;
    loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProcessError {
                kind: ProcessErrorKind::Canceled,
                message: "Operação cancelada".into(),
            });
        }
        if started.elapsed() >= spec.timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProcessError {
                kind: ProcessErrorKind::Timeout,
                message: format!("{} excedeu o timeout", spec.tool),
            });
        }
        match child.try_wait() {
            Ok(Some(value)) => {
                status = value;
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                return Err(ProcessError {
                    kind: ProcessErrorKind::Failed,
                    message: error.to_string(),
                })
            }
        }
    }
    let stdout = stdout_reader
        .join()
        .map_err(|_| ProcessError {
            kind: ProcessErrorKind::Failed,
            message: "Falha ao coletar a saída do processo".into(),
        })?
        .map_err(|error| ProcessError {
            kind: ProcessErrorKind::Failed,
            message: error.to_string(),
        })?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| ProcessError {
            kind: ProcessErrorKind::Failed,
            message: "Falha ao coletar os erros do processo".into(),
        })?
        .map_err(|error| ProcessError {
            kind: ProcessErrorKind::Failed,
            message: error.to_string(),
        })?;
    let outcome = ProcessOutcome {
        tool: spec.tool,
        logical_command: sanitize(&spec.logical_command),
        duration_ms: started.elapsed().as_millis(),
        exit_code: status.code(),
        stdout,
        stderr: sanitize(&String::from_utf8_lossy(&stderr)),
    };
    if status.success() {
        Ok(outcome)
    } else {
        let failure = outcome.stderr.to_ascii_lowercase();
        let kind = if failure.contains("failed to load perl dll")
            || failure.contains("dll") && failure.contains("code 126")
        {
            ProcessErrorKind::MissingDependency
        } else {
            ProcessErrorKind::Failed
        };
        Err(ProcessError {
            kind,
            message: format!(
                "{} terminou com código {:?}: {}",
                outcome.tool, outcome.exit_code, outcome.stderr
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    #[test]
    fn captures_output_without_shell() {
        #[cfg(windows)]
        let result = run(
            ProcessSpec::new("cmd", "cmd.exe").args(["/d", "/c", "echo lumina"]),
            &CancellationToken::default(),
        )
        .unwrap();
        #[cfg(not(windows))]
        let result = run(
            ProcessSpec::new("printf", "printf").args(["lumina"]),
            &CancellationToken::default(),
        )
        .unwrap();
        assert!(String::from_utf8_lossy(&result.stdout).contains("lumina"));
        assert!(!result.logical_command.is_empty());
        assert!(result.duration_ms < 30_000);
    }
    #[test]
    fn redacts_secrets() {
        assert_eq!(
            sanitize("ffmpeg token=abc file.mov password=x"),
            "ffmpeg [REDACTED] file.mov [REDACTED]"
        )
    }
    #[test]
    fn canceled_before_spawn() {
        let token = CancellationToken::default();
        token.cancel();
        assert_eq!(
            run(ProcessSpec::new("x", "none"), &token).unwrap_err().kind,
            ProcessErrorKind::Canceled
        )
    }
    #[test]
    fn missing_dependency_is_actionable() {
        let e = run(
            ProcessSpec::new("Teste", "lumina-command-that-does-not-exist"),
            &CancellationToken::default(),
        )
        .unwrap_err();
        assert_eq!(e.kind, ProcessErrorKind::MissingDependency);
        assert!(e.message.contains("PATH"))
    }
    #[test]
    #[cfg(windows)]
    fn dll_loader_failure_is_a_dependency_error() {
        let error = run(
            ProcessSpec::new("Teste", "cmd.exe").args([
                "/d", "/c",
                "echo Failed to load Perl DLL perl532.dll code 126 1>&2 & exit /b 1",
            ]),
            &CancellationToken::default(),
        )
        .unwrap_err();
        assert_eq!(error.kind, ProcessErrorKind::MissingDependency);
    }
    #[test]
    fn bundled_tool_resolution_does_not_rewrite_explicit_paths() {
        let explicit = if cfg!(windows) {
            r"C:\tools\ffmpeg.exe"
        } else {
            "/tools/ffmpeg"
        };
        assert_eq!(
            resolve_program(OsStr::new(explicit)),
            OsString::from(explicit)
        );
    }
    #[test]
    fn timeout_stops_process() {
        #[cfg(windows)]
        let spec = ProcessSpec::new("cmd", "cmd.exe")
            .args(["/d", "/c", "ping -n 5 127.0.0.1 >nul"])
            .timeout(Duration::from_millis(50));
        #[cfg(not(windows))]
        let spec = ProcessSpec::new("sleep", "sleep")
            .args(["2"])
            .timeout(Duration::from_millis(50));
        assert_eq!(
            run(spec, &CancellationToken::default()).unwrap_err().kind,
            ProcessErrorKind::Timeout
        )
    }
    #[test]
    #[cfg(windows)]
    fn drains_large_output_without_deadlock() {
        let result = run(
            ProcessSpec::new("PowerShell", "powershell.exe")
                .args([
                    "-NoProfile",
                    "-Command",
                    "[Console]::Out.Write('x'*1048576)",
                ])
                .timeout(Duration::from_secs(15)),
            &CancellationToken::default(),
        )
        .unwrap();
        assert_eq!(result.stdout.len(), 1_048_576);
    }
    #[test]
    #[cfg(windows)]
    fn cancels_a_running_process() {
        let token = CancellationToken::default();
        let worker_token = token.clone();
        let started = Arc::new(AtomicUsize::new(0));
        let worker_started = started.clone();
        let worker = std::thread::spawn(move || {
            worker_started.store(1, AtomicOrdering::SeqCst);
            run(
                ProcessSpec::new("PowerShell", "powershell.exe").args([
                    "-NoProfile",
                    "-Command",
                    "Start-Sleep -Seconds 10",
                ]),
                &worker_token,
            )
        });
        while started.load(AtomicOrdering::SeqCst) == 0 {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(100));
        token.cancel();
        assert_eq!(
            worker.join().unwrap().unwrap_err().kind,
            ProcessErrorKind::Canceled
        );
    }
    #[test]
    #[cfg(windows)]
    fn enforces_global_process_concurrency_limit() {
        TEST_PEAK_ACTIVE.store(0, Ordering::SeqCst);
        let workers = (0..6)
            .map(|_| {
                std::thread::spawn(|| {
                    run(
                        ProcessSpec::new("PowerShell", "powershell.exe").args([
                            "-NoProfile",
                            "-Command",
                            "Start-Sleep -Milliseconds 200",
                        ]),
                        &CancellationToken::default(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap();
        }
        assert_eq!(TEST_PEAK_ACTIVE.load(Ordering::SeqCst), PROCESS_LIMIT);
    }
}
