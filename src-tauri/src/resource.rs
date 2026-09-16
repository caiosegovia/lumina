use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Condvar, Mutex, OnceLock,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Interactive,
    Background,
}
struct State {
    active: usize,
    interactive_waiters: usize,
}
pub struct Gate {
    limit: AtomicUsize,
    state: Mutex<State>,
    wake: Condvar,
}
pub struct Permit(&'static Gate);
impl Drop for Permit {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        state.active = state.active.saturating_sub(1);
        self.0.wake.notify_all();
    }
}
impl Gate {
    fn acquire_with_cancel(
        &'static self,
        priority: Priority,
        cancel: Option<&crate::process::CancellationToken>,
    ) -> Option<Permit> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if priority == Priority::Interactive {
            state.interactive_waiters += 1;
        }
        while state.active >= self.limit.load(Ordering::Relaxed)
            || (priority == Priority::Background && state.interactive_waiters > 0)
        {
            if cancel.is_some_and(|token| token.is_cancelled()) {
                if priority == Priority::Interactive {
                    state.interactive_waiters = state.interactive_waiters.saturating_sub(1);
                }
                self.wake.notify_all();
                return None;
            }
            state = self
                .wake
                .wait_timeout(state, std::time::Duration::from_millis(50))
                .unwrap_or_else(|error| error.into_inner())
                .0;
        }
        if priority == Priority::Interactive {
            state.interactive_waiters -= 1;
        }
        state.active += 1;
        Some(Permit(self))
    }
    #[cfg(test)]
    fn acquire(&'static self, priority: Priority) -> Permit {
        self.acquire_with_cancel(priority, None)
            .expect("aquisição sem cancelamento")
    }
}
static IO: OnceLock<Gate> = OnceLock::new();
static PROFILE_LIMIT: AtomicUsize = AtomicUsize::new(2);
static PRESSURE_LEVEL: AtomicUsize = AtomicUsize::new(0);
fn io_gate() -> &'static Gate {
    IO.get_or_init(|| Gate {
        limit: AtomicUsize::new(2),
        state: Mutex::new(State {
            active: 0,
            interactive_waiters: 0,
        }),
        wake: Condvar::new(),
    })
}
pub fn set_profile(profile: &str) {
    let limit = match profile {
        "economy" => 1,
        "performance" => 4,
        _ => 2,
    };
    PROFILE_LIMIT.store(limit, Ordering::Relaxed);
    io_gate().limit.store(
        if PRESSURE_LEVEL.load(Ordering::Relaxed) > 0 {
            1
        } else {
            limit
        },
        Ordering::Relaxed,
    );
    io_gate().wake.notify_all();
}
pub fn set_pressure(level: usize) {
    PRESSURE_LEVEL.store(level.min(2), Ordering::Relaxed);
    let effective = if level > 0 {
        1
    } else {
        PROFILE_LIMIT.load(Ordering::Relaxed)
    };
    io_gate().limit.store(effective, Ordering::Relaxed);
    io_gate().wake.notify_all();
}
pub fn pressure_level() -> usize {
    PRESSURE_LEVEL.load(Ordering::Relaxed)
}
#[cfg(test)]
pub fn io(priority: Priority) -> Permit {
    io_gate().acquire(priority)
}
pub fn io_cancel(
    priority: Priority,
    cancel: &crate::process::CancellationToken,
) -> Result<Permit, String> {
    io_gate()
        .acquire_with_cancel(priority, Some(cancel))
        .ok_or_else(|| "JOB_CANCELED".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    #[test]
    fn global_io_limit_is_enforced() {
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let workers = (0..8)
            .map(|_| {
                let active = active.clone();
                let peak = peak.clone();
                std::thread::spawn(move || {
                    let _permit = io(Priority::Background);
                    let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(15));
                    active.fetch_sub(1, Ordering::SeqCst);
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().unwrap()
        }
        assert!(peak.load(Ordering::SeqCst) <= 2);
    }

    #[test]
    fn interactive_io_precedes_background_waiters() {
        use std::sync::mpsc;
        let gate: &'static Gate = Box::leak(Box::new(Gate {
            limit: AtomicUsize::new(1),
            state: Mutex::new(State {
                active: 0,
                interactive_waiters: 0,
            }),
            wake: Condvar::new(),
        }));
        let held = gate.acquire(Priority::Background);
        let (tx, rx) = mpsc::channel();
        let background = std::thread::spawn({
            let tx = tx.clone();
            move || {
                let _permit = gate.acquire(Priority::Background);
                tx.send("background").unwrap();
            }
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        let interactive = std::thread::spawn(move || {
            let _permit = gate.acquire(Priority::Interactive);
            tx.send("interactive").unwrap();
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        drop(held);
        assert_eq!(rx.recv().unwrap(), "interactive");
        interactive.join().unwrap();
        background.join().unwrap();
    }

    #[test]
    fn queued_io_observes_cancellation() {
        let gate: &'static Gate = Box::leak(Box::new(Gate {
            limit: AtomicUsize::new(1),
            state: Mutex::new(State {
                active: 0,
                interactive_waiters: 0,
            }),
            wake: Condvar::new(),
        }));
        let held = gate.acquire(Priority::Background);
        let token = crate::process::CancellationToken::default();
        let waiting = token.clone();
        let worker = std::thread::spawn(move || {
            gate.acquire_with_cancel(Priority::Background, Some(&waiting))
                .is_none()
        });
        std::thread::sleep(std::time::Duration::from_millis(25));
        token.cancel();
        assert!(worker.join().unwrap());
        drop(held);
    }
}
