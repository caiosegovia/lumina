use crate::process::CancellationToken;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
#[cfg(test)]
pub static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub running: bool,
    pub stage: String,
    pub completed: usize,
    pub total: usize,
    pub cancelled: bool,
}
#[derive(Default)]
struct State {
    progress: Progress,
    token: CancellationToken,
}
fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::default()))
}
pub fn progress() -> Progress {
    state()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .progress
        .clone()
}
pub fn cancel() {
    let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
    if s.progress.running {
        s.token.cancel();
        s.progress.cancelled = true;
    }
}
pub struct Work {
    pub token: CancellationToken,
}
impl Work {
    pub fn begin(stage: &str) -> Result<Self, String> {
        let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
        if s.progress.running {
            return Err("Já existe uma análise do Descobrir em execução".into());
        }
        s.token = CancellationToken::default();
        s.progress = Progress {
            running: true,
            stage: stage.into(),
            ..Progress::default()
        };
        Ok(Self {
            token: s.token.clone(),
        })
    }
    pub fn update(&self, completed: usize, total: usize) {
        let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
        s.progress.completed = completed;
        s.progress.total = total;
    }
}
impl Drop for Work {
    fn drop(&mut self) {
        state()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .progress
            .running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_releases_single_flight_and_preserves_committed_progress() {
        let _test = TEST_LOCK.lock().unwrap();
        let work = Work::begin("test").unwrap();
        work.update(2, 5);
        assert!(Work::begin("duplicate").is_err());
        cancel();
        assert!(work.token.is_cancelled());
        assert!(progress().running);
        drop(work);
        assert!(!progress().running);
        assert!(progress().cancelled);
        assert_eq!(progress().completed, 2);
        let resumed = Work::begin("resumed").unwrap();
        assert!(!resumed.token.is_cancelled());
        drop(resumed);
    }
    #[test]
    fn status_is_serialized_with_stable_contract() {
        let json = serde_json::to_value(Progress::default()).unwrap();
        assert_eq!(json["running"], false);
        assert_eq!(json["completed"], 0);
    }
}
