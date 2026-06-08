---
id: cap-source-daemon
summary: Source replay payload for projects/cap/src/daemon.rs
fill_sections: [source, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The cap source group implements command wrapping, daemon leases, throttling, and structured run outcomes."
  - id: agent-hook-installation
    role: primary
    gap: claude-and-codex-hook-installation
    claim: claude-and-codex-hook-installation
    coverage: full
    rationale: "The cap source group contains the installer logic for Claude Code and Codex CLI hook registration."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "The same source group contains hook rewrite and hook installation adapters."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: full
    rationale: "The sampler and throttle modules in this source group implement memory and CPU pressure sampling."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: daemon-process-lifecycle
    claim: daemon-process-lifecycle
    coverage: full
    rationale: "The daemon, status, wait, and ping surfaces are implemented in this source group."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: full
    rationale: "The CLI module in this source group implements status and wait command surfaces."
  - id: config-logging-and-reap-policy
    role: primary
    gap: configuration-defaults-and-compatibility
    claim: configuration-defaults-and-compatibility
    coverage: full
    rationale: "Configuration, JSONL logging, and reap policy modules live in this source group."
  - id: config-logging-and-reap-policy
    role: primary
    gap: run-log-persistence
    claim: run-log-persistence
    coverage: full
    rationale: "The event log module in this source group implements JSONL run-log persistence."
  - id: config-logging-and-reap-policy
    role: primary
    gap: reap-allowlist-policy
    claim: reap-allowlist-policy
    coverage: full
    rationale: "The reap module in this source group implements bounded allowlist-based process reaping."
---

# Source TD: projects/cap/src/daemon.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `is_running` | public Rust symbol in `projects/cap/src/daemon.rs` |
| `run_foreground` | public Rust symbol in `projects/cap/src/daemon.rs` |
| `spawn_background` | public Rust symbol in `projects/cap/src/daemon.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
//! Daemon process: owns the Throttle, runs the sampler loop, serves
//! UDS connections.
//!
//! Wire format: each connection is a stream of newline-delimited JSON
//! requests. The daemon associates the connection with at most one
//! lease — on EOF, that lease's child (if any) is best-effort SIGKILLed
//! and the lease is dropped, so a crashed client cannot leave an
//! orphan paused process hanging around.

use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::config::Config;
use crate::eventlog::EventLog;
use crate::paths;
use crate::protocol::{LeaseId, Request, Response};
use crate::reap::Reaper;
use crate::sampler::{LoadSampler, MemorySampler, RssSampler};
use crate::throttle::{Throttle, TickAction};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Hard cap on how long any `wait_for_capacity` request can park a
/// daemon connection. Even if the client asks for 1 h, we wake the
/// connection at this point so it can decide whether to re-issue the
/// wait or give up. 5 min was picked to match typical CI-step budgets.
const SERVER_WAIT_CAP: std::time::Duration = std::time::Duration::from_secs(5 * 60);

/// One-shot "are we currently under pause pressure?" sample, used by
/// the Acquire backpressure path. Creates fresh samplers because the
/// sampler loop's are owned by another task and `MemorySampler` /
/// `LoadSampler` are cheap to spin up.
fn is_under_pressure_now(throttle: &Throttle) -> bool {
    let mut mem = MemorySampler::new();
    let load = LoadSampler::new().load_per_core();
    throttle.is_under_pause_pressure(mem.free_gb(), load)
}

/// Hold an exclusive `flock` on `~/.cap/cap.lock` for the lifetime
/// of the daemon. Drop = release. If another daemon already holds it,
/// `acquire_singleton_lock` returns Ok(None) so the caller can exit
/// cleanly without disturbing the live daemon's socket / pid file.
struct SingletonLock {
    // Held only for its Drop — closing the fd releases the flock.
    _file: std::fs::File,
}

fn acquire_singleton_lock() -> Result<Option<SingletonLock>> {
    use std::os::unix::io::AsRawFd;
    paths::ensure_home()?;
    let path = paths::lock_path()?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        // The lockfile is purely an flock handle — we never write to it,
        // so keep any existing content rather than truncating.
        .truncate(false)
        .open(&path)
        .with_context(|| format!("opening lockfile {}", path.display()))?;
    // LOCK_NB makes flock return EWOULDBLOCK instead of waiting when
    // another process holds the lock — we want to exit, not queue.
    let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if rc == 0 {
        Ok(Some(SingletonLock { _file: file }))
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
            Ok(None)
        } else {
            Err(err).with_context(|| format!("flock {}", path.display()))
        }
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub async fn run_foreground() -> Result<()> {
    let _singleton = match acquire_singleton_lock()? {
        Some(l) => l,
        None => {
            tracing::info!("another cap daemon is already running; exiting");
            return Ok(());
        }
    };
    let cfg = Config::load()?;
    // Resolve TWO percentage caps into TWO absolute floors using
    // total RAM measured at startup. Take the larger of each pair so
    // small machines keep the explicit reserve and large machines
    // aren't forced to leave double-digit GB idle. Slice 1 plumbs
    // both floors through Throttle::new; Slice 2 introduces the
    // two-threshold tick logic that actually consults kill_floor.
    let mut probe = MemorySampler::new();
    let total_gb = probe.total_gb();
    let free_gb = probe.free_gb();
    let used_gb = (total_gb - free_gb).max(0.0);
    let absolute_floor = cfg.protect.min_free_gb;
    let pause_pct = cfg.protect.pause_used_percent.min(100) as f64;
    let kill_pct = cfg.protect.kill_used_percent.min(100) as f64;
    let derived_pause_floor = total_gb * (1.0 - pause_pct / 100.0);
    let derived_kill_floor = total_gb * (1.0 - kill_pct / 100.0);
    let pause_floor_gb = absolute_floor.max(derived_pause_floor);
    let kill_floor_gb = absolute_floor.max(derived_kill_floor);
    tracing::info!(
        total_gb = format!("{total_gb:.2}"),
        used_gb = format!("{used_gb:.2}"),
        free_gb = format!("{free_gb:.2}"),
        absolute_floor_gb = format!("{absolute_floor:.2}"),
        pause_used_percent = cfg.protect.pause_used_percent,
        kill_used_percent = cfg.protect.kill_used_percent,
        derived_pause_floor_gb = format!("{derived_pause_floor:.2}"),
        derived_kill_floor_gb = format!("{derived_kill_floor:.2}"),
        pause_floor_gb = format!("{pause_floor_gb:.2}"),
        kill_floor_gb = format!("{kill_floor_gb:.2}"),
        "memory floors resolved",
    );
    let eventlog = Arc::new(EventLog::new(cfg.log.enabled));
    let throttle = Throttle::new(cfg, pause_floor_gb, kill_floor_gb, total_gb)
        .context("invalid protect config: kill_used_percent must be > pause_used_percent")?;
    let socket = paths::socket_path()?;
    paths::ensure_home()?;
    let _ = std::fs::remove_file(&socket);

    let listener = UnixListener::bind(&socket)
        .with_context(|| format!("binding UDS at {}", socket.display()))?;
    tracing::info!(socket = %socket.display(), "cap daemon listening");
    std::fs::write(paths::pid_path()?, std::process::id().to_string())?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Spawn the sampler loop.
    let sampler_throttle = throttle.clone();
    let sampler_handle = tokio::spawn(sampler_loop(sampler_throttle));

    loop {
        tokio::select! {
            res = listener.accept() => {
                match res {
                    Ok((stream, _)) => {
                        let t = throttle.clone();
                        let stx = shutdown_tx.clone();
                        let el = eventlog.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_conn(stream, t, stx, el).await {
                                tracing::warn!(error = %e, "connection ended with error");
                            }
                        });
                    }
                    Err(e) => tracing::error!(error = %e, "accept failed"),
                }
            }
            _ = shutdown_rx.recv() => {
                tracing::info!("shutdown requested");
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("ctrl-c received");
                break;
            }
        }
    }

    sampler_handle.abort();
    let _ = std::fs::remove_file(&socket);
    let _ = std::fs::remove_file(paths::pid_path()?);
    Ok(())
}

async fn sampler_loop(throttle: Arc<Throttle>) {
    let interval =
        std::time::Duration::from_millis(throttle.config().protect.sample_interval_ms.max(50));
    let mut sampler = MemorySampler::new();
    let mut rss_sampler = RssSampler::new();
    let load_sampler = LoadSampler::new();
    let mut reaper = Reaper::new();
    let kill_floor = throttle.kill_floor_gb();
    loop {
        let free = sampler.free_gb();
        let load = load_sampler.load_per_core();
        // Refresh RSS only for our active lease PIDs (typically ≤ 8).
        // sysinfo's `refresh_processes_specifics` skips the full table
        // scan, so this is cheap even on busy boxes.
        let pids = throttle.active_child_pids().await;
        let rss_map = rss_sampler.rss_bytes(&pids);
        let rss_lookup: &(dyn Fn(i32) -> Option<u64> + Send + Sync) =
            &|pid| rss_map.get(&pid).copied();
        let action = throttle.tick(free, load, rss_lookup).await;

        // Reap path (Slice 6): when memory is below kill_floor and
        // configured, SIGTERM known auto-restarting non-lease
        // processes (LSPs, proc-macro servers). Runs in parallel to
        // the throttle tick — they target disjoint sets (lease PIDs
        // vs allowlisted non-lease PIDs). Cooldown bounds the
        // sysinfo full-refresh cost when memory stays low for long.
        let protect = &throttle.config().protect;
        if protect.reap_enabled
            && free < kill_floor
            && reaper.cooldown_elapsed(std::time::Duration::from_secs(protect.reap_cooldown_secs))
        {
            let reaped = reaper.scan_and_reap(protect.reap_min_uptime_secs, &pids);
            for r in &reaped {
                tracing::warn!(
                    pid = r.pid,
                    name = %r.name,
                    rss_mb = r.rss_bytes / (1024 * 1024),
                    uptime_secs = r.uptime_secs,
                    free_gb = free,
                    "reaped allowlisted process (auto-restarting non-lease)"
                );
            }
        }
        match action {
            TickAction::Idle => {}
            TickAction::Resumed(id) => {
                tracing::info!(lease = id, free_gb = free, "resumed");
            }
            TickAction::PausedNewest(id) => {
                tracing::info!(
                    lease = id,
                    free_gb = free,
                    "paused-newest (memory pressure)"
                );
            }
            TickAction::TermedVictim { id, classification } => {
                tracing::warn!(
                    lease = id,
                    free_gb = free,
                    classification = ?classification,
                    grace_secs = throttle.config().protect.kill_grace_secs,
                    "termed-victim (sent SIGTERM, waiting grace)"
                );
            }
            TickAction::EscalatedToKill { id } => {
                tracing::warn!(
                    lease = id,
                    free_gb = free,
                    "escalated-to-kill (grace expired, SIGKILL group)"
                );
            }
            TickAction::KilledVictim { id, classification } => {
                tracing::warn!(
                    lease = id,
                    free_gb = free,
                    classification = ?classification,
                    "killed-victim (memory pressure, no grace)"
                );
            }
            TickAction::KilledAllPaused(ids) => {
                tracing::warn!(
                    free_gb = free,
                    leases = ?ids,
                    "killed-all-paused (last resort)"
                );
            }
        }
        tokio::time::sleep(interval).await;
    }
}

async fn handle_conn(
    stream: UnixStream,
    throttle: Arc<Throttle>,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
    eventlog: Arc<EventLog>,
) -> Result<()> {
    let (rx, mut tx) = stream.into_split();
    let mut reader = BufReader::new(rx);
    let mut line = String::new();

    // Lease (if any) owned by this connection. Released on EOF.
    let mut held_lease: Option<LeaseId> = None;

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let req: Request = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                write_resp(
                    &mut tx,
                    &Response::Error {
                        message: format!("bad request: {e}"),
                    },
                )
                .await?;
                continue;
            }
        };

        match req {
            Request::Ping => {
                write_resp(
                    &mut tx,
                    &Response::Pong {
                        version: VERSION.into(),
                    },
                )
                .await?;
            }
            Request::Acquire(a) => {
                let label = a.label.clone().unwrap_or_else(|| {
                    if a.args.is_empty() {
                        a.program.clone()
                    } else {
                        format!("{} {}", a.program, a.args.join(" "))
                    }
                });
                // Strategy hints in the kill envelope (and the run log's
                // `program`) are keyed by the bare program basename. For
                // hook-wrapped `bash -c '<script>'` invocations we look
                // through the shell to the real tool (`cargo`, `pytest`),
                // so hints fire and the log groups by the actual command.
                let program = crate::hook::effective_program(&a.program, &a.args);
                // Acquire-time backpressure: if the system is currently
                // under pause pressure (memory OR CPU), wait for
                // headroom before handing out the lease. Server-side
                // hard cap of 5 min ensures we never hang a client
                // forever. The client perceives this as `cap run`
                // simply taking longer to start.
                if is_under_pressure_now(&throttle) {
                    let _ = throttle.wait_for_capacity(Some(SERVER_WAIT_CAP)).await;
                }
                let lease = throttle.register(a.client_pid, program, label, a.cwd).await;
                held_lease = Some(lease);
                let nice = throttle.config().defaults.nice;
                write_resp(&mut tx, &Response::Lease { lease, nice }).await?;
            }
            Request::Spawned { lease, child_pid } => {
                throttle.attach_pid(lease, child_pid).await;
                write_resp(&mut tx, &Response::SpawnedAck).await?;
            }
            Request::Release { lease, exit_code } => {
                let outcome = throttle.release(lease, exit_code).await;
                if let Some(rec) = &outcome.record {
                    eventlog.append(rec);
                }
                if held_lease == Some(lease) {
                    held_lease = None;
                }
                write_resp(
                    &mut tx,
                    &Response::Released {
                        lease,
                        kill_envelope: outcome.kill_envelope,
                    },
                )
                .await?;
            }
            Request::Status | Request::Ps => {
                let mut sampler = MemorySampler::new();
                let free = sampler.free_gb();
                let load_sampler = LoadSampler::new();
                let load = load_sampler.load_per_core();
                let snap = throttle.snapshot(free, load).await;
                write_resp(&mut tx, &Response::Status(snap)).await?;
            }
            Request::WaitForCapacity { timeout_secs } => {
                // Server-side hard cap of 5 minutes regardless of what
                // the client requested, so a runaway client cannot
                // pin a connection forever. `None` lets the cap apply;
                // `Some(0)` is a non-blocking probe.
                let deadline = match timeout_secs {
                    None => SERVER_WAIT_CAP,
                    Some(0) => std::time::Duration::from_secs(0),
                    Some(s) => std::time::Duration::from_secs(s).min(SERVER_WAIT_CAP),
                };
                let ok = throttle.wait_for_capacity(Some(deadline)).await;
                let resp = if ok {
                    Response::CapacityOk
                } else {
                    Response::CapacityTimeout
                };
                write_resp(&mut tx, &resp).await?;
            }
            Request::Shutdown => {
                write_resp(&mut tx, &Response::ShuttingDown).await?;
                let _ = shutdown_tx.send(()).await;
                break;
            }
        }
    }

    // EOF safety net: drop the lease so a crashed client cannot leave
    // its child paused forever. The command still gets a run-log record
    // (exit code unknown — the client never sent Release).
    if let Some(lease) = held_lease.take() {
        let outcome = throttle.release(lease, None).await;
        if let Some(rec) = &outcome.record {
            eventlog.append(rec);
        }
    }
    Ok(())
}

async fn write_resp<W: AsyncWriteExt + Unpin>(w: &mut W, r: &Response) -> Result<()> {
    let mut s = serde_json::to_string(r)?;
    s.push('\n');
    w.write_all(s.as_bytes()).await?;
    Ok(())
}

/// Is a daemon currently running for this user?
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn is_running() -> Result<bool> {
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

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub fn spawn_background() -> Result<i32> {
    use std::process::{Command, Stdio};
    paths::ensure_home()?;
    let exe = std::env::current_exe().context("locating cap binary")?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths::log_path()?)?;
    let log_err = log.try_clone()?;
    let child = Command::new(exe)
        .args(["daemon", "run"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err))
        .spawn()
        .context("spawning cap daemon")?;
    Ok(child.id() as i32)
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/daemon.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#rust_source.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-daemon-rs-source-replay-superseded>"
```
