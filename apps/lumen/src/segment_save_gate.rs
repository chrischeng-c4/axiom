//! Root-local checkpoint serialization with an owned permit. Acquisition may
//! block and therefore belongs outside every apply/capture lease. The permit
//! can move from preflight to an export worker without releasing the root.

use std::sync::{Arc, Condvar, Mutex};

#[derive(Default)]
pub(super) struct SaveGate {
    held: Mutex<bool>,
    released: Condvar,
}

#[must_use = "dropping the permit releases checkpoint serialization"]
pub(super) struct SavePermit {
    gate: Arc<SaveGate>,
}

impl SaveGate {
    pub(super) fn lock_owned(self: &Arc<Self>) -> SavePermit {
        let mut held = self.held.lock().unwrap_or_else(|p| p.into_inner());
        while *held {
            held = self.released.wait(held).unwrap_or_else(|p| p.into_inner());
        }
        *held = true;
        SavePermit { gate: self.clone() }
    }
}

impl Drop for SavePermit {
    fn drop(&mut self) {
        *self.gate.held.lock().unwrap_or_else(|p| p.into_inner()) = false;
        self.gate.released.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::SaveGate;
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn owned_permit_retains_exclusion_until_drop_after_cross_thread_move() {
        let gate = Arc::new(SaveGate::default());
        let permit = gate.lock_owned();
        let second = gate.clone();
        let (attempt_tx, attempt_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            attempt_tx.send(()).unwrap();
            let _permit = second.lock_owned();
            done_tx.send(()).unwrap();
        });
        attempt_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let acquired_early = done_rx.recv_timeout(Duration::from_millis(50)).is_ok();
        drop(permit);
        if !acquired_early {
            done_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        }
        worker.join().unwrap();
        assert!(
            !acquired_early,
            "second owner entered before retained permit dropped"
        );
    }

    #[test]
    fn owned_permit_can_move_to_its_drop_thread() {
        let gate = Arc::new(SaveGate::default());
        let permit = gate.lock_owned();
        let (dropped_tx, dropped_rx) = mpsc::channel();
        std::thread::spawn(move || {
            drop(permit);
            dropped_tx.send(()).unwrap();
        })
        .join()
        .unwrap();
        dropped_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let second = gate.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _next = second.lock_owned();
            done_tx.send(()).unwrap();
        });
        done_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn dropping_owner_after_an_error_path_frees_the_gate() {
        let gate = Arc::new(SaveGate::default());
        let failed = || -> Result<(), ()> {
            let _permit = gate.lock_owned();
            Err(())
        };
        assert_eq!(failed(), Err(()));
        let second = gate.clone();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _next = second.lock_owned();
            done_tx.send(()).unwrap();
        });
        done_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.join().unwrap();
    }
}
