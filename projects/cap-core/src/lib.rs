//! cap-core — the client side of cap, factored out so any tool can register
//! its child processes as cap leases.
//!
//! cap protects a machine from memory pressure with **one** daemon that watches
//! OS free memory and SIGSTOP/SIGCONT/SIGKILLs the commands it manages. The
//! value is that single, system-wide arbiter — so the reusable piece is not the
//! throttling *policy* (that stays in the daemon, in the `cap` binary) but the
//! *client integration*: speak the [`protocol`], [`connect`](client::Client)
//! to the daemon, and run a child through [`managed_run`](managed_run::managed_run)
//! so it becomes a lease.
//!
//! Consumers:
//! - the `cap` CLI (`cap run`), and
//! - `vat`, which runs each sandboxed workload as a cap lease and folds the
//!   returned [`KillEnvelope`](protocol::KillEnvelope) into its agent-legible
//!   state.
//!
//! A second arbiter must never be embedded — that would mean two processes
//! fighting over the same global free-memory signal.

pub mod client;
pub mod managed_run;
pub mod paths;
pub mod protocol;
pub mod supervisor;

pub use client::Client;
pub use managed_run::{managed_run, ManagedOutcome};

/// Is a cap daemon currently running for this user? Reads the pidfile and
/// probes the pid with `kill(pid, 0)`. Pure — no daemon dependency — so the
/// client can use it to decide whether to launch one.
pub fn is_running() -> anyhow::Result<bool> {
    let pid_path = paths::pid_path()?;
    if !pid_path.exists() {
        return Ok(false);
    }
    let pid_text = std::fs::read_to_string(&pid_path)?;
    let pid: i32 = pid_text.trim().parse().unwrap_or(0);
    if pid <= 0 {
        return Ok(false);
    }
    let alive = unsafe { libc::kill(pid, 0) == 0 };
    Ok(alive)
}
