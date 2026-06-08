//! End-to-end crash-recovery tests: real subprocess + SIGKILL / `libc::abort`.
//!
//! Complements `persistence_test.rs` (in-process Drop-based) by exercising the
//! realistic abrupt-termination path the WAL + snapshot layer is engineered
//! against. Protocol: parent forks, child opens engine + runs ops + (maybe)
//! flushes, signals a ready byte over a pipe, then `libc::abort()`s or idles
//! until parent SIGKILLs. Parent `waitpid`s (asserts killed-by-signal), then
//! reopens on the same data_dir and checks invariants.
//!
//! Determinism (R4): fixed N=100, pipe-based ready sync, ≤10s hard timeout (R5).
//! Guarded by `#[cfg(unix)]`; Windows → single ignored stub (R7).

#[cfg(not(unix))]
#[test]
#[ignore = "crash recovery via fork/SIGKILL requires POSIX"]
fn crash_recovery_windows_not_supported() {}

#[cfg(unix)]
mod unix_impl {

    use std::os::unix::io::RawFd;
    use std::path::Path;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    use cclab_ctx_inf_db::{CtxInfEngine, Entity, EntityType, PersistenceConfig, RecoveryManager};
    use nix::sys::signal::{kill, Signal};
    use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
    use nix::unistd::{close, fork, read, write, ForkResult, Pid};
    use tempfile::TempDir;

    const N: usize = 100;
    const WAIT_TIMEOUT: Duration = Duration::from_secs(10);

    /// Read one ready byte from `fd` or fail on timeout / EOF / error.
    fn read_ready(fd: RawFd, deadline: Instant) -> Result<(), String> {
        let mut buf = [0u8; 1];
        while Instant::now() < deadline {
            match read(fd, &mut buf) {
                Ok(1) => return Ok(()),
                Ok(0) => return Err("pipe EOF before ready byte".into()),
                Ok(_) => unreachable!(),
                Err(nix::errno::Errno::EINTR) => continue,
                Err(e) => return Err(format!("pipe read: {e}")),
            }
        }
        Err("timed out waiting for child ready byte".into())
    }

    /// Poll waitpid with WNOHANG until child dies or deadline — no CI hangs.
    fn wait_with_timeout(pid: Pid, deadline: Instant) -> Result<WaitStatus, String> {
        loop {
            match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::StillAlive) => {
                    if Instant::now() >= deadline {
                        let _ = kill(pid, Signal::SIGKILL);
                        let _ = waitpid(pid, None);
                        return Err("subprocess did not exit within timeout".into());
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Ok(status) => return Ok(status),
                Err(nix::errno::Errno::EINTR) => continue,
                Err(e) => return Err(format!("waitpid: {e}")),
            }
        }
    }

    fn assert_killed_by_signal(status: WaitStatus) {
        match status {
            WaitStatus::Signaled(_, sig, _) => assert!(
                sig == Signal::SIGKILL || sig == Signal::SIGABRT,
                "expected SIGKILL/SIGABRT, got {sig:?}"
            ),
            other => panic!("expected Signaled(..), got {other:?} — hard-crash path not taken"),
        }
    }

    /// Fork. Child runs `child_body(write_fd)` (ends by aborting or looping).
    /// Parent waits for ready byte, optionally SIGKILLs, asserts killed-by-signal,
    /// then runs `verify(data_dir)`.
    fn run_crash_scenario<C, V>(data_dir: &Path, parent_sigkills: bool, child_body: C, verify: V)
    where
        C: FnOnce(RawFd) + Send + 'static,
        V: FnOnce(&Path),
    {
        let (read_fd, write_fd) = nix::unistd::pipe().expect("pipe");

        // SAFETY: fork() before any worker thread is spawned — child starts
        // single-threaded, so async-signal-safe ops + libc::abort() are safe.
        match unsafe { fork() }.expect("fork") {
            ForkResult::Child => {
                let _ = close(read_fd);
                child_body(write_fd);
                // Distinct exit code so parent can tell the body returned instead of aborting.
                unsafe { nix::libc::_exit(77) };
            }
            ForkResult::Parent { child } => {
                let _ = close(write_fd);
                let deadline = Instant::now() + WAIT_TIMEOUT;
                if let Err(e) = read_ready(read_fd, deadline) {
                    let _ = kill(child, Signal::SIGKILL);
                    let _ = waitpid(child, None);
                    let _ = close(read_fd);
                    panic!("ready-byte sync failed: {e}");
                }
                if parent_sigkills {
                    kill(child, Signal::SIGKILL).expect("kill SIGKILL");
                }
                let status = wait_with_timeout(child, deadline).expect("waitpid");
                let _ = close(read_fd);
                assert_killed_by_signal(status);
                verify(data_dir);
            }
        }
    }

    fn send_ready(fd: RawFd) {
        let _ = write(fd, &[1u8]);
        let _ = close(fd);
    }

    // ── R2a: SIGKILL after flush → expect exactly N entities ─────────────

    #[test]
    fn test_recovery_after_sigkill_with_flush() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().to_path_buf();
        let child_dir = data_dir.clone();
        run_crash_scenario(
            &data_dir,
            false,
            move |w| {
                let engine =
                    CtxInfEngine::with_persistence(PersistenceConfig::for_testing(&child_dir))
                        .expect("child: with_persistence");
                for i in 0..N {
                    engine
                        .create_entity(Entity::new(EntityType::Person, format!("P{i}")))
                        .expect("child: create_entity");
                }
                engine.flush();
                // Production flush() is async try_send — sleep lets background thread drain + fsync.
                thread::sleep(Duration::from_millis(500));
                send_ready(w);
                unsafe { nix::libc::abort() };
            },
            |data_dir| {
                let (engine, stats) = RecoveryManager::recover(data_dir).expect("parent: recover");
                assert_eq!(
                    engine.stats().entity_count,
                    N,
                    "post-flush abort must preserve exactly N={N} (wal_replayed={}, corrupted={})",
                    stats.wal_entries_replayed,
                    stats.corrupted_entries
                );
            },
        );
    }

    // ── R2b: SIGKILL without flush → 0..=N, no crash, no corruption ──────

    #[test]
    fn test_recovery_after_sigkill_without_flush() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().to_path_buf();
        let child_dir = data_dir.clone();
        run_crash_scenario(
            &data_dir,
            false,
            move |w| {
                let engine =
                    CtxInfEngine::with_persistence(PersistenceConfig::for_testing(&child_dir))
                        .expect("child: with_persistence");
                for i in 0..N {
                    engine
                        .create_entity(Entity::new(EntityType::Person, format!("Q{i}")))
                        .expect("child: create_entity");
                }
                // No flush — whatever the 10ms periodic background flush catches is all we get.
                send_ready(w);
                unsafe { nix::libc::abort() };
            },
            |data_dir| {
                let (engine, _s) =
                    RecoveryManager::recover(data_dir).expect("parent: recover must not crash");
                let count = engine.stats().entity_count;
                assert!(count <= N, "recovered count {count} exceeds N={N}");
                // Every recovered entity must have valid id+type (no corruption).
                for e in engine.entities_by_type(&EntityType::Person) {
                    assert!(e.name.starts_with('Q'), "corrupt name: {:?}", e.name);
                    assert_ne!(e.id.0.as_u128(), 0, "recovered entity has nil id");
                }
            },
        );
    }

    // ── R2c: SIGKILL mid-snapshot → atomic-rename invariant holds ────────

    #[test]
    fn test_recovery_after_sigkill_mid_snapshot() {
        let temp = TempDir::new().unwrap();
        let data_dir = temp.path().to_path_buf();
        let child_dir = data_dir.clone();
        run_crash_scenario(
            &data_dir,
            true,
            move |w| {
                let engine =
                    CtxInfEngine::with_persistence(PersistenceConfig::for_testing(&child_dir))
                        .expect("child: with_persistence");
                for i in 0..50 {
                    engine
                        .create_entity(Entity::new(EntityType::Person, format!("S{i}")))
                        .expect("child: create_entity");
                }
                engine.flush();
                thread::sleep(Duration::from_millis(100));

                // Snapshot in a background thread; parent SIGKILLs us anywhere during
                // the `.tmp → .snap` rename window.
                let engine = Arc::new(engine);
                let snap_engine = engine.clone();
                let started = Arc::new(AtomicBool::new(false));
                let started_clone = started.clone();
                let _snap = thread::spawn(move || {
                    started_clone.store(true, Ordering::SeqCst);
                    let _ = snap_engine.create_snapshot();
                });
                while !started.load(Ordering::SeqCst) {
                    thread::yield_now();
                }
                send_ready(w);
                // Idle until SIGKILL — normal exit would fail assert_killed_by_signal.
                loop {
                    thread::sleep(Duration::from_millis(10));
                }
            },
            |data_dir| {
                // Recovery MUST NOT crash. SHA-256 verification inside load_latest enforces
                // the atomic-rename invariant: stray `.tmp` files are ignored.
                let (engine, _s) =
                    RecoveryManager::recover(data_dir).expect("parent: recover must not crash");
                assert!(
                    engine.stats().entity_count <= 50,
                    "count exceeds 50 inserts"
                );
            },
        );
    }
} // mod unix_impl
