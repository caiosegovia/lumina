use std::sync::{Condvar, Mutex, OnceLock};

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
    limit: usize,
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
    fn acquire(&'static self, priority: Priority) -> Permit {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if priority == Priority::Interactive {
            state.interactive_waiters += 1;
        }
        while state.active >= self.limit
            || (priority == Priority::Background && state.interactive_waiters > 0)
        {
            state = self.wake.wait(state).unwrap_or_else(|e| e.into_inner());
        }
        if priority == Priority::Interactive {
            state.interactive_waiters -= 1;
        }
        state.active += 1;
        Permit(self)
    }
}
static IO: OnceLock<Gate> = OnceLock::new();
pub fn io(priority: Priority) -> Permit {
    IO.get_or_init(|| Gate {
        limit: 2,
        state: Mutex::new(State {
            active: 0,
            interactive_waiters: 0,
        }),
        wake: Condvar::new(),
    })
    .acquire(priority)
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
            limit: 1,
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
}
