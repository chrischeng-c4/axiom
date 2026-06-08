//! The supervised-run helper: run a child as a cap *lease* so the daemon can
//! throttle it under memory pressure, and hand back the structured outcome.
//!
//! This is the reusable core of what was `cap run`'s `handle_run`. The cap CLI
//! and vat both call it; neither re-implements the Acquire → Spawned → wait →
//! Release dance, the process-group spawn, or the terminal-signal forwarding.
//!
//! The daemon stays the single arbiter: callers register here as clients, they
//! do **not** embed throttling logic of their own.

use anyhow::{bail, Result};

use crate::client::Client;
use crate::protocol::{AcquireRequest, KillEnvelope, Request, Response};
use crate::supervisor::{self, SpawnOpts, SpawnSpec};

/// Result of a supervised run.
pub struct ManagedOutcome {
    /// The child's final exit status.
    pub status: std::process::ExitStatus,
    /// Present iff the daemon SIGKILLed the child under memory pressure.
    /// Read `classification` / `action` to decide whether to retry.
    pub kill_envelope: Option<KillEnvelope>,
}

/// Run `spec` under daemon protection on an already-connected `client`.
///
/// `label` is the human/agent-facing name recorded in the lease (e.g. the
/// original command line, or `vat:<id>`). The full lease dance:
///
/// 1. `Acquire` — register intent, receive a lease id + suggested `nice`.
/// 2. spawn the child in its own process group.
/// 3. `Spawned` — tell the daemon the PID so it can target it.
/// 4. wait (the daemon may pause/resume/kill behind our back).
/// 5. `Release` — report exit; receive a kill envelope iff we were killed.
pub async fn managed_run(
    client: &mut Client,
    spec: SpawnSpec,
    label: Option<String>,
) -> Result<ManagedOutcome> {
    let cwd = spec
        .cwd
        .clone()
        .or_else(|| std::env::current_dir().ok())
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    // 1. Register intent.
    let acquire = Request::Acquire(AcquireRequest {
        program: spec.program.clone(),
        args: spec.args.clone(),
        cwd,
        label,
        client_pid: std::process::id() as i32,
    });
    let (lease, nice) = match client.request(&acquire).await? {
        Response::Lease { lease, nice } => (lease, nice),
        Response::Error { message } => bail!("cap daemon: {message}"),
        other => bail!("unexpected daemon response to Acquire: {other:?}"),
    };

    // 2. Spawn the child in its own process group.
    let running = supervisor::spawn(&spec, SpawnOpts { nice })?;
    let child_pid = running.pid;

    // 2a. Forward terminal-driven signals to the child's group. The supervisor
    //     remapped the process group so the throttler can target the subtree;
    //     the side effect is the terminal no longer delivers Ctrl-C — restore
    //     that by hand.
    let forwarder = spawn_signal_forwarder(child_pid);

    // 3. Tell the daemon the PID.
    match client
        .request(&Request::Spawned { lease, child_pid })
        .await?
    {
        Response::SpawnedAck => {}
        Response::Error { message } => eprintln!("cap: warn: daemon rejected Spawned: {message}"),
        other => eprintln!("cap: warn: unexpected response to Spawned: {other:?}"),
    }

    // 4. Wait for the child; the daemon may pause/kill it behind our back.
    let status = running.wait().await?;
    forwarder.abort();

    // 5. Release and collect the kill envelope (if any).
    let kill_envelope = match client
        .request(&Request::Release {
            lease,
            exit_code: status.code(),
        })
        .await
    {
        Ok(Response::Released { kill_envelope, .. }) => kill_envelope,
        _ => None,
    };

    Ok(ManagedOutcome {
        status,
        kill_envelope,
    })
}

/// Forward SIGINT/SIGTERM/SIGHUP to the child's process group so an
/// interactive Ctrl-C still reaches it despite the process-group remap.
fn spawn_signal_forwarder(child_pid: i32) -> tokio::task::JoinHandle<()> {
    use tokio::signal::unix::{signal, SignalKind};
    tokio::spawn(async move {
        let (mut sigint, mut sigterm, mut sighup) = match (
            signal(SignalKind::interrupt()),
            signal(SignalKind::terminate()),
            signal(SignalKind::hangup()),
        ) {
            (Ok(a), Ok(b), Ok(c)) => (a, b, c),
            _ => return,
        };
        loop {
            let sig = tokio::select! {
                _ = sigint.recv()  => libc::SIGINT,
                _ = sigterm.recv() => libc::SIGTERM,
                _ = sighup.recv()  => libc::SIGHUP,
            };
            unsafe {
                libc::kill(-child_pid, sig);
            }
        }
    })
}
