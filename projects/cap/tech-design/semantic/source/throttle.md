---
id: cap-source-throttle
summary: Source replay payload for projects/cap/src/throttle.rs
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

# Source TD: projects/cap/src/throttle.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `Lease` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `ReleaseOutcome` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `RssLookup` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `Throttle` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `TickAction` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `active_child_pids` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `attach_pid` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `config` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `is_under_pause_pressure` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `kill_floor_gb` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `new` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `pause_floor_gb` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `register` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `release` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `snapshot` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `tick` | public Rust symbol in `projects/cap/src/throttle.rs` |
| `wait_for_capacity` | public Rust symbol in `projects/cap/src/throttle.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
//! Live throttler: pauses / resumes / kills cap-managed children
//! based on the OS's "available memory" reading and (Slice 5) load
//! per CPU core.
//!
//! Two-threshold decision per tick:
//!
//! ```text
//!   free ≥ pause_floor              → resume oldest paused
//!   kill_floor ≤ free < pause_floor → pause newest running
//!                                     (incl. solo; kept alive)
//!   free < kill_floor               → select_victim (largest by RSS,
//!                                     paused first then running),
//!                                     SIGKILL with classification.
//!                                     After N consecutive ticks
//!                                     still below kill_floor →
//!                                     SIGKILL every paused lease.
//! ```
//!
//! RSS is supplied by the caller via a closure so tests can fake it;
//! Slice 3 wires `sysinfo`'s `processes` feature for real readings.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, Notify};

use crate::config::Config;
use crate::eventlog::RunRecord;
use crate::protocol::{
    Action, KillClassification, KillEnvelope, LeaseBrief, LeaseId, LeaseSnapshot, LeaseState,
    StatusSnapshot,
};

/// Closure that returns per-process RSS in bytes, or `None` if the
/// PID is unknown / dead. Slice 2 callers pass `|_| None`; Slice 3
/// passes a `sysinfo`-backed lookup. `Send + Sync` is required so
/// `tick()` can be awaited from the tokio sampler task.
pub type RssLookup<'a> = &'a (dyn Fn(i32) -> Option<u64> + Send + Sync);

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug)]
pub struct Lease {
    pub id: LeaseId,
    pub client_pid: i32,
    pub child_pid: Option<i32>,
    /// Bare program name (argv[0], basename only) used to pick a
    /// strategy hint in the kill envelope. The `label` field is for
    /// human display and may carry the full argv.
    pub program: String,
    pub label: String,
    /// Working directory the command was launched in (for the run log).
    pub cwd: String,
    pub registered: Instant,
    /// Wall-clock submission time, for the run log's `started_at`.
    pub registered_wall: chrono::DateTime<chrono::Local>,
    /// When the client reported the child PID (the `Spawned` step). The
    /// gap `spawned_at - registered` is the queue time the run log
    /// reports. `None` until the command actually starts.
    pub spawned_at: Option<Instant>,
    pub state: LeaseState,
    /// Peak leader-process RSS seen across the run (run-log metric,
    /// updated each tick from the sampler's RSS lookup).
    pub peak_rss_bytes: u64,
    /// System free memory captured on the first tick after the child
    /// started — the run log's `free_gb_at_start`.
    pub free_gb_at_start: Option<f64>,
    /// Accumulated time this lease has spent SIGSTOPped, plus the start
    /// of the current pause if it's paused right now. Combined at
    /// release time into the run log's `paused_ms`.
    pub paused_total: Duration,
    pub paused_since: Option<Instant>,
    /// Set when the throttler kills the child. Surfaced to the
    /// client on Release so it can render a structured diagnostic.
    pub kill_envelope: Option<KillEnvelope>,
    /// When the throttler first sent SIGTERM (transition Running/Paused
    /// → Killing). Used by the grace-period escalator on subsequent
    /// ticks: once `elapsed >= kill_grace_secs`, escalate to a
    /// process-group SIGKILL. Uses `tokio::time::Instant` so tests can
    /// drive it via `tokio::time::pause + advance`.
    pub kill_started_at: Option<tokio::time::Instant>,
}

#[derive(Debug)]
struct State {
    next_id: LeaseId,
    leases: HashMap<LeaseId, Lease>,
    /// Consecutive sub-pause-floor samples (debounce for pause path).
    sub_threshold_run: u32,
    /// Consecutive sub-kill-floor samples (debounce for kill +
    /// last-resort kill-all-paused path).
    kill_floor_run: u32,
    /// Whether the previous tick saw memory- or CPU-side pause pressure.
    /// Slice 5 uses the `true → false` edge to fire `headroom_ok.notify_waiters()`
    /// exactly once per recovery, so `cap wait` clients (and Acquire-time
    /// backpressure in daemon.rs) wake without polling.
    was_under_pressure: bool,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct Throttle {
    cfg: Config,
    /// Memory free-GB floor that triggers pause (back-pressure +
    /// SIGSTOP-newest). Derived from `pause_used_percent` and
    /// `min_free_gb` at daemon startup. Always > `kill_floor_gb`.
    pause_floor_gb: f64,
    /// Memory free-GB floor that triggers kill (victim eviction).
    /// Derived from `kill_used_percent` and `min_free_gb` at daemon
    /// startup. Always < `pause_floor_gb`.
    kill_floor_gb: f64,
    /// Total system RAM in GB, measured once at daemon startup. Used
    /// to compute `KillClassification::Oversize` (single-lease RSS
    /// exceeds total - kill_floor headroom).
    total_gb: f64,
    /// Edge-triggered notifier: fires `notify_waiters` once per clean
    /// transition from "under pause pressure" → "headroom OK", so
    /// `cap wait` (and Acquire-time backpressure in daemon.rs) can
    /// resume without polling. Not fired every tick — only on the edge.
    headroom_ok: Notify,
    state: Mutex<State>,
}

/// Result of releasing a lease: the kill diagnostic (if cap killed it)
/// plus the run-log record (if the command ever actually started).
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug)]
pub struct ReleaseOutcome {
    pub kill_envelope: Option<KillEnvelope>,
    pub record: Option<RunRecord>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone)]
pub enum TickAction {
    Idle,
    Resumed(LeaseId),
    /// Newest running lease was SIGSTOPped because free dipped below
    /// pause_floor. Slice 2 keeps the FIFO-newest policy even when RSS
    /// is known; victim selection by RSS only kicks in at kill time.
    PausedNewest(LeaseId),
    /// First-stage kill (Slice 4): SIGTERM sent to the lease leader
    /// (NOT the process group), state transitioned to `Killing`. The
    /// daemon waits `kill_grace_secs` then escalates to a group
    /// SIGKILL. The envelope was built at this step and stored on the
    /// lease so the client sees the same diagnostic regardless of
    /// which stage actually freed the RSS.
    TermedVictim {
        id: LeaseId,
        classification: KillClassification,
    },
    /// Grace-period escalation (Slice 4): kill_grace_secs elapsed
    /// since SIGTERM, process group SIGKILLed and state moved to
    /// `Killed`. Surfaces no new classification — the original kill
    /// envelope is still authoritative.
    EscalatedToKill {
        id: LeaseId,
    },
    /// One-shot SIGKILL (no grace) under kill-floor pressure. Selection
    /// order: largest paused by RSS → largest running by RSS. Reached
    /// when `kill_grace_secs == 0` (skip-grace config).
    KilledVictim {
        id: LeaseId,
        classification: KillClassification,
    },
    /// Last resort: still below kill_floor after
    /// `kill_all_paused_after_ticks` consecutive ticks even after
    /// per-tick victim kills. SIGKILL every paused lease in one shot —
    /// grace is intentionally skipped (host is at OOM imminence).
    KilledAllPaused(Vec<LeaseId>),
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Throttle {
    /// Build a throttler with both protection floors already resolved
    /// (daemon does the derivation from percentages + absolute floor).
    /// Returns an error if the invariant `kill_floor_gb < pause_floor_gb`
    /// is violated — the daemon should refuse to start in that case.
    // The negated comparisons below are deliberate: `!(a < b)` and
    // `!(x > 0.0)` also reject NaN floors (a plain `>=` / `<=` would let
    // a NaN through), so the daemon refuses to start on a degenerate
    // config instead of running with undefined thresholds.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn new(
        cfg: Config,
        pause_floor_gb: f64,
        kill_floor_gb: f64,
        total_gb: f64,
    ) -> anyhow::Result<Arc<Self>> {
        if !(kill_floor_gb < pause_floor_gb) {
            anyhow::bail!(
                "kill_floor_gb ({kill_floor_gb:.2}) must be strictly less than \
                 pause_floor_gb ({pause_floor_gb:.2}); check kill_used_percent > \
                 pause_used_percent in config",
            );
        }
        if !(total_gb > 0.0) {
            anyhow::bail!("total_gb must be positive, got {total_gb}");
        }
        Ok(Arc::new(Self {
            cfg,
            pause_floor_gb,
            kill_floor_gb,
            total_gb,
            headroom_ok: Notify::new(),
            state: Mutex::new(State {
                next_id: 1,
                leases: HashMap::new(),
                sub_threshold_run: 0,
                kill_floor_run: 0,
                was_under_pressure: false,
            }),
        }))
    }

    pub fn pause_floor_gb(&self) -> f64 {
        self.pause_floor_gb
    }

    pub fn kill_floor_gb(&self) -> f64 {
        self.kill_floor_gb
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// Register a new lease in `Pending` state. Returns its id.
    /// `program` is the bare argv[0] (basename) — used at kill time to
    /// pick a strategy hint for the kill envelope. `label` is for human
    /// display (typically `program + args`).
    pub async fn register(
        &self,
        client_pid: i32,
        program: String,
        label: String,
        cwd: String,
    ) -> LeaseId {
        let mut st = self.state.lock().await;
        let id = st.next_id;
        st.next_id = st.next_id.wrapping_add(1).max(1);
        st.leases.insert(
            id,
            Lease {
                id,
                client_pid,
                child_pid: None,
                program,
                label,
                cwd,
                registered: Instant::now(),
                registered_wall: chrono::Local::now(),
                spawned_at: None,
                state: LeaseState::Pending,
                peak_rss_bytes: 0,
                free_gb_at_start: None,
                paused_total: Duration::ZERO,
                paused_since: None,
                kill_envelope: None,
                kill_started_at: None,
            },
        );
        id
    }

    /// Snapshot of currently-live child PIDs (any state with a real
    /// `child_pid`). The sampler loop uses this to scope the per-tick
    /// RSS refresh to just our lease set.
    pub async fn active_child_pids(&self) -> Vec<i32> {
        let st = self.state.lock().await;
        st.leases
            .values()
            .filter_map(|l| l.child_pid)
            .filter(|p| *p > 0)
            .collect()
    }

    /// Client tells us the PID it just spawned. From this point on,
    /// the throttler may target the process with SIGSTOP / SIGCONT /
    /// SIGKILL.
    pub async fn attach_pid(&self, lease: LeaseId, child_pid: i32) {
        let mut st = self.state.lock().await;
        if let Some(l) = st.leases.get_mut(&lease) {
            l.child_pid = Some(child_pid);
            if l.spawned_at.is_none() {
                l.spawned_at = Some(Instant::now());
            }
            if l.state == LeaseState::Pending {
                l.state = LeaseState::Running;
            }
        }
    }

    /// Drop a lease entirely. Returns the `kill_envelope` (if the
    /// throttler had killed the child, so the client can surface a
    /// structured diagnostic on stderr) plus a run-log record for the
    /// command, built from the lease's accumulated bookkeeping.
    /// `exit_code` is the child's exit status as the client observed it.
    pub async fn release(&self, lease: LeaseId, exit_code: Option<i32>) -> ReleaseOutcome {
        let mut st = self.state.lock().await;
        let Some(l) = st.leases.remove(&lease) else {
            return ReleaseOutcome {
                kill_envelope: None,
                record: None,
            };
        };
        let record = build_run_record(&l, exit_code);
        ReleaseOutcome {
            kill_envelope: l.kill_envelope,
            record,
        }
    }

    /// True if either axis (memory or CPU load) is in the pause zone.
    /// Pure function over the sampled scalars — no state access. Used
    /// by Acquire-time backpressure in daemon.rs to decide whether to
    /// block before registering a new lease, and by the tick body for
    /// edge-detecting headroom recovery.
    pub fn is_under_pause_pressure(&self, free_mem_gb: f64, load_per_core: f64) -> bool {
        // No upper clamp: load can legitimately exceed nproc, so a floor
        // above 100% (e.g. 150 = 1.5x cores) is a valid config.
        let load_pause_floor = self.cfg.protect.pause_load_percent as f64 / 100.0;
        free_mem_gb < self.pause_floor_gb
            || (load_pause_floor > 0.0 && load_per_core > load_pause_floor)
    }

    /// Block until both memory and CPU are above their pause floors,
    /// or until `timeout` elapses. Returns `true` for capacity-ok,
    /// `false` for timeout. Server-side hard cap is enforced by the
    /// caller (daemon.rs) — this method honors the timeout it was
    /// given, no more.
    pub async fn wait_for_capacity(&self, timeout: Option<std::time::Duration>) -> bool {
        // Fast path: if a tick recently observed not-under-pressure,
        // return immediately. `was_under_pressure` is the freshest
        // signal we have; consult it before parking on the Notify.
        {
            let st = self.state.lock().await;
            if !st.was_under_pressure {
                return true;
            }
        }
        match timeout {
            None => {
                self.headroom_ok.notified().await;
                true
            }
            Some(d) if d.is_zero() => {
                // Non-blocking probe; we already saw "under pressure"
                // above, so it's a timeout.
                false
            }
            Some(d) => matches!(
                tokio::time::timeout(d, self.headroom_ok.notified()).await,
                Ok(())
            ),
        }
    }

    /// Run one throttle tick against `free_mem_gb` and `load_per_core`.
    /// `rss_of` is used at kill time to pick the largest victim and to
    /// classify the kill; tests pass `&|_| None`, production code
    /// passes a sysinfo lookup (Slice 3). Returns the action taken so
    /// callers (tests, logs) can see what happened. CPU is pause-only
    /// — it never contributes to the kill path.
    pub async fn tick(
        &self,
        free_mem_gb: f64,
        load_per_core: f64,
        rss_of: RssLookup<'_>,
    ) -> TickAction {
        let pause_floor = self.pause_floor_gb;
        let kill_floor = self.kill_floor_gb;
        let load_pause_floor = self.cfg.protect.pause_load_percent as f64 / 100.0;
        let cpu_over = load_pause_floor > 0.0 && load_per_core > load_pause_floor;
        let trigger_samples = self.cfg.protect.trigger_samples.max(1);
        let kill_all_after = self.cfg.protect.kill_all_paused_after_ticks.max(1);
        let grace = std::time::Duration::from_secs(self.cfg.protect.kill_grace_secs);
        let under_pause_pressure = free_mem_gb < pause_floor || cpu_over;
        let mut st = self.state.lock().await;

        // Edge-trigger headroom_ok on every `under → ok` transition.
        // Done before any state mutation so the notify happens even on
        // the resume-path tick that wakes the oldest paused lease.
        let prev_under = st.was_under_pressure;
        st.was_under_pressure = under_pause_pressure;
        if prev_under && !under_pause_pressure {
            self.headroom_ok.notify_waiters();
        }

        // ── Run-log bookkeeping ────────────────────────────────────
        // Track per-lease peak RSS and the free memory seen just after
        // each command started. Cheap (≤ a handful of live leases) and
        // feeds the JSONL record built at release time.
        for l in st.leases.values_mut() {
            if let Some(pid) = l.child_pid {
                if let Some(rss) = rss_of(pid) {
                    l.peak_rss_bytes = l.peak_rss_bytes.max(rss);
                }
                if l.free_gb_at_start.is_none() {
                    l.free_gb_at_start = Some(free_mem_gb);
                }
            }
        }

        // ── Grace-period escalator ─────────────────────────────────
        // Runs FIRST every tick, regardless of headroom, so we don't
        // leave SIGTERM'd leases hanging if pressure clears between
        // the SIGTERM and the SIGKILL. Pick the oldest expired
        // Killing lease — multiple-in-one-tick is rare, and surfacing
        // one per tick keeps logs readable.
        let now = tokio::time::Instant::now();
        let expired = st
            .leases
            .values()
            .filter(|l| l.state == LeaseState::Killing)
            .filter(|l| {
                l.kill_started_at
                    .map(|t| now.duration_since(t) >= grace)
                    .unwrap_or(true) // defensive: missing timestamp → escalate
            })
            .min_by_key(|l| l.kill_started_at.unwrap_or(now))
            .map(|l| (l.id, l.child_pid));
        if let Some((id, pid)) = expired {
            if let Some(pid) = pid {
                send_signal(pid, libc::SIGKILL); // process group, this time
            }
            if let Some(l) = st.leases.get_mut(&id) {
                l.state = LeaseState::Killed;
            }
            return TickAction::EscalatedToKill { id };
        }

        // ── Above pause floor AND CPU ok: pressure clear ──────────
        if free_mem_gb >= pause_floor && !cpu_over {
            st.sub_threshold_run = 0;
            st.kill_floor_run = 0;
            let oldest_paused = st
                .leases
                .values()
                .filter(|l| l.state == LeaseState::Paused)
                .min_by_key(|l| l.registered)
                .map(|l| (l.id, l.child_pid));
            if let Some((id, Some(pid))) = oldest_paused {
                send_signal(pid, libc::SIGCONT);
                if let Some(l) = st.leases.get_mut(&id) {
                    l.state = LeaseState::Running;
                    if let Some(since) = l.paused_since.take() {
                        l.paused_total += since.elapsed();
                    }
                }
                return TickAction::Resumed(id);
            }
            return TickAction::Idle;
        }

        // ── Under pause pressure: debounce noise ──────────────────
        st.sub_threshold_run = st.sub_threshold_run.saturating_add(1);
        if st.sub_threshold_run < trigger_samples {
            return TickAction::Idle;
        }

        // ── Pause zone (mem ≥ kill_floor, OR CPU-only pressure) ───
        // CPU never reaches the kill path — load is too coarse and
        // SIGSTOP fully releases CPU, so pausing is sufficient.
        if free_mem_gb >= kill_floor {
            // Sliding back out of the kill zone — reset the hard
            // floor debounce so we don't carry stale counts.
            st.kill_floor_run = 0;
            st.sub_threshold_run = 0;
            // Pause newest running lease (incl. solo). Pause-newest
            // preserves FIFO finish order. Killing-state leases are
            // excluded — they're already being torn down.
            let mut running: Vec<(LeaseId, Option<i32>, Instant)> = st
                .leases
                .values()
                .filter(|l| l.state == LeaseState::Running)
                .map(|l| (l.id, l.child_pid, l.registered))
                .collect();
            if running.is_empty() {
                return TickAction::Idle;
            }
            running.sort_by_key(|(_, _, t)| *t);
            let (id, pid, _) = running.last().copied().unwrap();
            if let Some(pid) = pid {
                send_signal(pid, libc::SIGSTOP);
            }
            if let Some(l) = st.leases.get_mut(&id) {
                l.state = LeaseState::Paused;
                if l.paused_since.is_none() {
                    l.paused_since = Some(Instant::now());
                }
            }
            return TickAction::PausedNewest(id);
        }

        // ── Kill zone (free < kill_floor) ─────────────────────────
        st.kill_floor_run = st.kill_floor_run.saturating_add(1);

        // Last-resort: been below kill_floor for too many ticks even
        // after per-tick victim kills. SIGKILL every paused lease in
        // one shot — they're all just sitting on RSS that's making
        // the host OOM. Memory comes back only when their pages are
        // freed.
        if st.kill_floor_run >= kill_all_after {
            let paused: Vec<(LeaseId, Option<i32>, String, String)> = st
                .leases
                .values()
                .filter(|l| l.state == LeaseState::Paused)
                .map(|l| (l.id, l.child_pid, l.program.clone(), l.label.clone()))
                .collect();
            if !paused.is_empty() {
                let run = st.kill_floor_run;
                let total_gb = self.total_gb;
                let other_briefs = brief_others(
                    &st,
                    &paused.iter().map(|(id, ..)| *id).collect::<Vec<_>>(),
                    rss_of,
                );
                let mut killed = Vec::with_capacity(paused.len());
                for (id, pid, program, label) in paused {
                    if let Some(pid) = pid {
                        send_signal(pid, libc::SIGKILL);
                    }
                    let victim_rss_gb = pid.and_then(rss_of).map(bytes_to_gb).unwrap_or(0.0);
                    let envelope = build_envelope(
                        KillClassification::Competition,
                        &program,
                        &label,
                        victim_rss_gb,
                        free_mem_gb,
                        kill_floor,
                        total_gb,
                        other_briefs.clone(),
                        Some(format!(
                            "last-resort kill-all-paused: free {free_mem_gb:.2} GB \
                             below kill_floor {kill_floor:.2} GB for \
                             {run} consecutive ticks",
                        )),
                    );
                    if let Some(l) = st.leases.get_mut(&id) {
                        l.state = LeaseState::Killed;
                        l.kill_envelope = Some(envelope);
                    }
                    killed.push(id);
                }
                st.kill_floor_run = 0;
                st.sub_threshold_run = 0;
                return TickAction::KilledAllPaused(killed);
            }
            // Nothing paused left — fall through to per-tick victim.
        }

        // Per-tick victim pick: largest paused by RSS first, then
        // largest running. Killing-state leases are excluded.
        let Some((victim_id, victim_pid, victim_rss)) = select_victim(&st, rss_of) else {
            // No cap-managed lease to act on; pressure is fully
            // external. Reset run so a recovered-then-relapsed
            // pattern doesn't trip the last-resort path spuriously.
            return TickAction::Idle;
        };

        // Classify BEFORE mutating state so the active count is
        // measured pre-kill (matches how the agent will read it).
        let active_lease_count = st
            .leases
            .values()
            .filter(|l| {
                matches!(
                    l.state,
                    LeaseState::Running | LeaseState::Paused | LeaseState::Killing
                )
            })
            .count();
        let victim_rss_gb = victim_rss as f64 / 1024.0 / 1024.0 / 1024.0;
        let headroom_budget_gb = (self.total_gb - kill_floor).max(0.0);
        let classification = classify_kill(victim_rss_gb, headroom_budget_gb, active_lease_count);

        // Snapshot victim metadata + sibling briefs before we mutate
        // state — the mutable borrow below would otherwise conflict
        // with the immutable iteration `brief_others` needs.
        let (victim_program, victim_label) = st
            .leases
            .get(&victim_id)
            .map(|l| (l.program.clone(), l.label.clone()))
            .unwrap_or_default();
        let other_briefs = brief_others(&st, &[victim_id], rss_of);
        let envelope = build_envelope(
            classification,
            &victim_program,
            &victim_label,
            victim_rss_gb,
            free_mem_gb,
            kill_floor,
            self.total_gb,
            other_briefs,
            None,
        );

        // Grace-period split (Slice 4):
        //   kill_grace_secs == 0  → SIGKILL whole group right now, one shot.
        //   kill_grace_secs >  0  → SIGTERM the leader only (give cargo /
        //                            pytest a chance to flush state), record
        //                            kill_started_at, transition to Killing.
        //                            Escalator at top of next tick promotes
        //                            it to a group SIGKILL once grace expires.
        let skip_grace = self.cfg.protect.kill_grace_secs == 0;
        // Don't reset kill_floor_run yet — successive ticks at kill
        // pressure should accumulate toward the last-resort path.
        st.sub_threshold_run = 0;

        if skip_grace {
            if let Some(pid) = victim_pid {
                send_signal(pid, libc::SIGKILL);
            }
            if let Some(l) = st.leases.get_mut(&victim_id) {
                l.state = LeaseState::Killed;
                l.kill_envelope = Some(envelope);
            }
            TickAction::KilledVictim {
                id: victim_id,
                classification,
            }
        } else {
            // SIGTERM the leader only — NOT the process group — so the
            // leader (cargo, pytest) gets a chance to coordinate its
            // children's cleanup before we escalate.
            if let Some(pid) = victim_pid {
                unsafe {
                    let _ = libc::kill(pid, libc::SIGTERM);
                }
            }
            if let Some(l) = st.leases.get_mut(&victim_id) {
                l.state = LeaseState::Killing;
                l.kill_started_at = Some(tokio::time::Instant::now());
                l.kill_envelope = Some(envelope);
            }
            TickAction::TermedVictim {
                id: victim_id,
                classification,
            }
        }
    }

    pub async fn snapshot(&self, free_mem_gb: f64, load_per_core: f64) -> StatusSnapshot {
        let st = self.state.lock().await;
        let mut running = 0u32;
        let mut paused = 0u32;
        let mut leases: Vec<LeaseSnapshot> = st
            .leases
            .values()
            .map(|l| {
                match l.state {
                    LeaseState::Running => running += 1,
                    LeaseState::Paused => paused += 1,
                    _ => {}
                }
                LeaseSnapshot {
                    lease: l.id,
                    client_pid: l.client_pid,
                    child_pid: l.child_pid,
                    label: l.label.clone(),
                    state: l.state,
                    age_secs: l.registered.elapsed().as_secs(),
                }
            })
            .collect();
        leases.sort_by_key(|l| l.lease);
        let load_pause_floor = self.cfg.protect.pause_load_percent as f64 / 100.0;
        StatusSnapshot {
            free_mem_gb,
            pause_floor_gb: self.pause_floor_gb,
            kill_floor_gb: self.kill_floor_gb,
            load_per_core,
            load_pause_floor,
            min_free_gb: self.pause_floor_gb,
            running,
            paused,
            leases,
        }
    }
}

/// Pick the largest victim by RSS — paused leases first (they're
/// already idle so killing them releases RSS without aborting useful
/// work), then running. Returns `(lease_id, child_pid, rss_bytes)`,
/// or `None` if there is no eligible victim. Excludes `Killing` /
/// `Killed` / `Pending` states. When RSS is `None` for every
/// candidate (Slice 2 with no real sysinfo), falls back to selecting
/// the newest by `registered`.
fn select_victim(st: &State, rss_of: RssLookup<'_>) -> Option<(LeaseId, Option<i32>, u64)> {
    let pick_max_rss = |target_state: LeaseState| -> Option<(LeaseId, Option<i32>, u64)> {
        let candidates: Vec<(&Lease, u64)> = st
            .leases
            .values()
            .filter(|l| l.state == target_state)
            .map(|l| {
                let rss = l.child_pid.and_then(rss_of).unwrap_or(0);
                (l, rss)
            })
            .collect();
        if candidates.is_empty() {
            return None;
        }
        // If we have any non-zero RSS, prefer max RSS. Otherwise fall
        // back to newest by registered time (FIFO-newest-first).
        let any_rss = candidates.iter().any(|(_, r)| *r > 0);
        let chosen = if any_rss {
            candidates.iter().max_by_key(|(_, r)| *r)
        } else {
            candidates.iter().max_by_key(|(l, _)| l.registered)
        }?;
        Some((chosen.0.id, chosen.0.child_pid, chosen.1))
    };
    pick_max_rss(LeaseState::Paused).or_else(|| pick_max_rss(LeaseState::Running))
}

/// Classify a kill so the agent knows what to do next:
/// - `Oversize`: a single lease's RSS exceeded the system's headroom
///   budget (`total_gb - kill_floor_gb`). Retrying the same command
///   will hit the same wall — agent must change strategy.
/// - `External`: only one active cap lease, and its RSS is small —
///   so non-cap processes (LSPs, browsers) are the real pressure.
///   Agent should `cap wait` and let the system recover.
/// - `Competition`: multiple cap leases were competing; the largest
///   was evicted to free room. Retry-after-wait works.
fn classify_kill(
    victim_rss_gb: f64,
    headroom_budget_gb: f64,
    active_lease_count: usize,
) -> KillClassification {
    // 256 MB = "small"; below this we assume the victim isn't the
    // real cause of pressure (likely external load).
    const SMALL_RSS_GB: f64 = 0.25;
    if headroom_budget_gb > 0.0 && victim_rss_gb > headroom_budget_gb {
        return KillClassification::Oversize;
    }
    if active_lease_count <= 1 && victim_rss_gb < SMALL_RSS_GB {
        return KillClassification::External;
    }
    KillClassification::Competition
}

fn send_signal(pid: i32, sig: libc::c_int) {
    unsafe {
        // Negative pid = "send to process group PGID=pid". The
        // supervisor makes each child its own group leader via
        // setpgid(0, 0), so this hits the whole subtree (cargo test
        // → rustc → ...). EPERM / ESRCH are OK to ignore — child may
        // have already exited or be in mid-reap.
        let _ = libc::kill(-pid, sig);
    }
}

fn bytes_to_gb(b: u64) -> f64 {
    b as f64 / 1024.0 / 1024.0 / 1024.0
}

/// Build the JSONL run-log record for a finished lease, or `None` if the
/// command never actually started (no `spawned_at` — e.g. the client
/// died during Acquire backpressure). Durations are computed from the
/// lease bookkeeping at release time.
fn build_run_record(l: &Lease, exit_code: Option<i32>) -> Option<RunRecord> {
    let spawned_at = l.spawned_at?;
    let now = Instant::now();
    let queue = spawned_at.saturating_duration_since(l.registered);
    let duration = now.saturating_duration_since(spawned_at);
    let mut paused = l.paused_total;
    if let Some(since) = l.paused_since {
        // Still paused at release (e.g. killed while SIGSTOPped) — count
        // the in-flight pause too.
        paused += now.saturating_duration_since(since);
    }
    let killed = l.kill_envelope.is_some();
    Some(RunRecord {
        ts: chrono::Local::now().to_rfc3339(),
        started_at: l.registered_wall.to_rfc3339(),
        lease: l.id,
        command: l.label.clone(),
        program: l.program.clone(),
        cwd: l.cwd.clone(),
        client_pid: l.client_pid,
        child_pid: l.child_pid,
        queue_ms: queue.as_millis() as u64,
        duration_ms: duration.as_millis() as u64,
        paused_ms: paused.as_millis() as u64,
        peak_rss_gb: bytes_to_gb(l.peak_rss_bytes),
        free_gb_at_start: l.free_gb_at_start,
        exit_code,
        outcome: if killed { "killed" } else { "completed" },
        kill_classification: l.kill_envelope.as_ref().map(|e| e.classification),
    })
}

/// Strategy hint keyed by argv[0] basename. Phrased as a *suggestion*
/// so agents who pipe `next_step` into an LLM are not given a directive.
fn strategy_hint(program: &str) -> &'static str {
    // Strip path prefix if any — supervisors may pass /usr/bin/cargo.
    let base = std::path::Path::new(program)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(program);
    match base {
        "cargo" => "lower `--jobs N` or build fewer crates",
        "pytest" => "run a subset with `-k <pattern>` or `--maxfail=N`",
        "npm" | "yarn" | "pnpm" => "lower `--max-parallel` or run scripts sequentially",
        "webpack" => "split build into smaller entry points or lower `--parallelism`",
        "tsc" => "use `--incremental` or split `tsconfig`",
        "go" => "lower `-p` (parallel package count) or build fewer packages",
        _ => "lower parallelism or split the workload",
    }
}

/// Snapshot of every lease *other than* the ones we're about to kill,
/// at the moment of the kill. Used for `KillEnvelope::other_leases` so
/// the agent can see who was competing for memory.
fn brief_others(st: &State, exclude: &[LeaseId], rss_of: RssLookup<'_>) -> Vec<LeaseBrief> {
    st.leases
        .values()
        .filter(|l| !exclude.contains(&l.id))
        .filter(|l| {
            matches!(
                l.state,
                LeaseState::Running | LeaseState::Paused | LeaseState::Killing
            )
        })
        .map(|l| LeaseBrief {
            lease: l.id,
            label: l.label.clone(),
            state: l.state,
            rss_gb: l.child_pid.and_then(rss_of).map(bytes_to_gb).unwrap_or(0.0),
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn build_envelope(
    classification: KillClassification,
    victim_program: &str,
    victim_label: &str,
    victim_rss_gb: f64,
    free_gb: f64,
    kill_floor_gb: f64,
    total_gb: f64,
    other_leases: Vec<LeaseBrief>,
    extra_note: Option<String>,
) -> KillEnvelope {
    let action = match classification {
        KillClassification::Competition => Action::WaitAndRetry {
            suggested_secs: 30,
            next_step: format!(
                "cap wait && {victim_label}    # the largest sibling will finish first"
            ),
        },
        KillClassification::Oversize => Action::ChangeStrategy {
            hint: strategy_hint(victim_program).to_string(),
            next_step: format!(
                "# retrying as-is will hit the same wall — {}",
                strategy_hint(victim_program)
            ),
        },
        KillClassification::External => Action::InspectAndWait {
            suggested_secs: 60,
            next_step: format!(
                "# non-cap RAM pressure; consider `ps -o rss,command -ax | sort -nr | head` \
                 then `cap wait && {victim_label}`"
            ),
        },
    };

    let mut human = String::new();
    use std::fmt::Write as _;
    let _ = writeln!(
        human,
        "cap: killed `{victim_label}` — {label}",
        label = classification_label(classification),
    );
    let _ = writeln!(
        human,
        "  victim RSS {victim_rss_gb:.2} GB; free {free_gb:.2} GB; \
         kill_floor {kill_floor_gb:.2} GB; total {total_gb:.2} GB"
    );
    if !other_leases.is_empty() {
        let _ = writeln!(human, "  siblings at kill time:");
        for b in &other_leases {
            let _ = writeln!(
                human,
                "    [{state:?}] {label}  RSS {rss:.2} GB",
                state = b.state,
                label = b.label,
                rss = b.rss_gb,
            );
        }
    }
    let _ = writeln!(human, "  next: {}", action_next_step(&action));
    if let Some(note) = &extra_note {
        let _ = writeln!(human, "  note: {note}");
    }

    KillEnvelope {
        classification,
        action,
        victim_label: victim_label.to_string(),
        victim_rss_gb,
        free_gb,
        kill_floor_gb,
        total_gb,
        other_leases,
        human_message: human,
    }
}

fn classification_label(c: KillClassification) -> &'static str {
    match c {
        KillClassification::Competition => "memory competition (other cap leases were running)",
        KillClassification::Oversize => "single-process oversize (RSS exceeded headroom budget)",
        KillClassification::External => {
            "external pressure (non-cap processes are eating the budget)"
        }
    }
}

fn action_next_step(a: &Action) -> &str {
    match a {
        Action::WaitAndRetry { next_step, .. }
        | Action::ChangeStrategy { next_step, .. }
        | Action::InspectAndWait { next_step, .. } => next_step.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with(min_free: f64, trigger: u32) -> Config {
        let mut c = Config::default();
        c.protect.min_free_gb = min_free;
        c.protect.trigger_samples = trigger;
        // Default-off grace: the bulk of throttle tests exercise the
        // immediate-SIGKILL path so they stay synchronous. Slice 4
        // grace tests construct a Config inline with grace > 0.
        c.protect.kill_grace_secs = 0;
        // CPU pause ships off-by-default; the CPU-path tests below want
        // an active floor (0.8 = 80% of nproc), so pin it here. The
        // `..._zero` test overrides this back to 0 explicitly.
        c.protect.pause_load_percent = 80;
        c
    }

    /// Test helper: pause_floor = min_free, kill_floor = min_free - 0.1
    /// (kept distinct so the invariant holds), total_gb = 16.0.
    fn throttle_with(min_free: f64, trigger: u32) -> Arc<Throttle> {
        Throttle::new(
            cfg_with(min_free, trigger),
            min_free,
            (min_free - 0.1).max(0.01),
            16.0,
        )
        .expect("test floors satisfy invariant")
    }

    /// No-op RSS lookup: every PID is unknown. Used by tests that
    /// don't care about victim selection beyond FIFO fallback.
    const NO_RSS: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|_| None;

    async fn add_running(t: &Throttle, label: &str) -> LeaseId {
        let id = t
            .register(1, label.into(), label.into(), String::new())
            .await;
        // i32::MAX is virtually guaranteed not to exist; libc::kill
        // returns ESRCH which we ignore.
        t.attach_pid(id, i32::MAX).await;
        id
    }

    async fn add_running_pid(t: &Throttle, label: &str, pid: i32) -> LeaseId {
        let id = t
            .register(1, label.into(), label.into(), String::new())
            .await;
        t.attach_pid(id, pid).await;
        id
    }

    #[tokio::test]
    async fn no_action_when_pressure_ok() {
        let t = throttle_with(2.0, 1);
        let _ = add_running(&t, "a").await;
        let _ = add_running(&t, "b").await;
        assert!(matches!(t.tick(8.0, 0.0, NO_RSS).await, TickAction::Idle));
    }

    #[tokio::test]
    async fn pauses_newest_under_pressure() {
        let t = throttle_with(2.0, 1);
        let a = add_running(&t, "a").await;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let b = add_running(&t, "b").await;
        // free=1.95 is in the pause zone (kill_floor=1.9, pause_floor=2.0).
        match t.tick(1.95, 0.0, NO_RSS).await {
            TickAction::PausedNewest(id) => assert_eq!(id, b),
            other => panic!("expected PausedNewest({b}), got {other:?}"),
        }
        let snap = t.snapshot(1.95, 0.0).await;
        let a_state = snap.leases.iter().find(|l| l.lease == a).unwrap().state;
        assert_eq!(a_state, LeaseState::Running);
    }

    #[tokio::test]
    async fn resumes_oldest_paused_when_pressure_clears() {
        let t = throttle_with(2.0, 1);
        let a = add_running(&t, "a").await;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let b = add_running(&t, "b").await;
        // Force b → paused (pause zone).
        t.tick(1.95, 0.0, NO_RSS).await;
        // Pressure clears.
        match t.tick(8.0, 0.0, NO_RSS).await {
            TickAction::Resumed(id) => assert_eq!(id, b),
            other => panic!("expected Resumed({b}), got {other:?}"),
        }
        let _ = a;
    }

    #[tokio::test]
    async fn solo_runner_above_kill_floor_pauses_not_kills() {
        // Regression: old single-floor logic killed solo runners as
        // soon as free dropped below the floor. New: solo runner
        // above kill_floor is paused (back-pressure); only below
        // kill_floor does the daemon escalate to a kill.
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let a = add_running(&t, "a").await;
        // free=1.5 is below pause_floor (2.0) but above kill_floor (1.0)
        // → must SIGSTOP, not SIGKILL (the bug we're guarding against).
        match t.tick(1.5, 0.0, NO_RSS).await {
            TickAction::PausedNewest(id) => assert_eq!(id, a),
            other => panic!("expected PausedNewest({a}), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn kills_solo_runner_below_kill_floor() {
        let t = throttle_with(2.0, 1); // kill_floor=1.9
        let a = add_running(&t, "a").await;
        // free=0.5 is below kill_floor → victim kill.
        match t.tick(0.5, 0.0, NO_RSS).await {
            TickAction::KilledVictim { id, classification } => {
                assert_eq!(id, a);
                // Solo lease + tiny RSS (NO_RSS → 0) → External.
                assert_eq!(classification, KillClassification::External);
            }
            other => panic!("expected KilledVictim, got {other:?}"),
        }
        let outcome = t.release(a, None).await;
        assert!(
            outcome.kill_envelope.is_some(),
            "kill reason must be surfaced on release"
        );
        // The command ran (was spawned), so it must also yield a record,
        // flagged as killed.
        let rec = outcome.record.expect("killed command still gets a record");
        assert_eq!(rec.outcome, "killed");
        assert_eq!(rec.kill_classification, Some(KillClassification::External));
    }

    #[tokio::test]
    async fn trigger_samples_debounces_noise() {
        let t = throttle_with(2.0, 3);
        let _ = add_running(&t, "a").await;
        let _ = add_running(&t, "b").await;
        // First two pause-zone samples → no action.
        assert!(matches!(t.tick(1.95, 0.0, NO_RSS).await, TickAction::Idle));
        assert!(matches!(t.tick(1.95, 0.0, NO_RSS).await, TickAction::Idle));
        // Third triggers.
        assert!(matches!(
            t.tick(1.95, 0.0, NO_RSS).await,
            TickAction::PausedNewest(_)
        ));
    }

    // ── Slice 2 specific tests ────────────────────────────────────

    #[tokio::test]
    async fn kill_picks_largest_paused_by_rss() {
        // Three leases, two paused with different RSS. select_victim
        // must prefer the LARGEST PAUSED first (before any running).
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let small_paused = add_running_pid(&t, "small_paused", 100).await;
        let big_paused = add_running_pid(&t, "big_paused", 200).await;
        let running = add_running_pid(&t, "running", 300).await;
        // Manually mark the first two as paused.
        {
            let mut st = t.state.lock().await;
            st.leases.get_mut(&small_paused).unwrap().state = LeaseState::Paused;
            st.leases.get_mut(&big_paused).unwrap().state = LeaseState::Paused;
        }
        // RSS: small=200MB, big=4GB, running=8GB.
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| match pid {
            100 => Some(200 * 1024 * 1024),
            200 => Some(4 * 1024 * 1024 * 1024),
            300 => Some(8 * 1024 * 1024 * 1024),
            _ => None,
        };
        match t.tick(0.5, 0.0, rss).await {
            TickAction::KilledVictim { id, .. } => assert_eq!(
                id, big_paused,
                "expected largest paused (big_paused) to be victim, got {id}"
            ),
            other => panic!("expected KilledVictim, got {other:?}"),
        }
        let _ = running;
    }

    #[tokio::test]
    async fn kill_falls_back_to_largest_running_when_none_paused() {
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let small = add_running_pid(&t, "small", 100).await;
        let big = add_running_pid(&t, "big", 200).await;
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| match pid {
            100 => Some(500 * 1024 * 1024),
            200 => Some(6 * 1024 * 1024 * 1024),
            _ => None,
        };
        match t.tick(0.5, 0.0, rss).await {
            TickAction::KilledVictim { id, classification } => {
                assert_eq!(id, big, "expected largest running to be victim");
                // Multiple active leases → Competition (not External).
                assert_eq!(classification, KillClassification::Competition);
            }
            other => panic!("expected KilledVictim, got {other:?}"),
        }
        let _ = small;
    }

    #[tokio::test]
    async fn last_resort_kills_all_paused_after_n_ticks() {
        // Two paused leases, free always below kill_floor, no running
        // leases. After kill_all_paused_after_ticks consecutive ticks
        // we must SIGKILL every paused.
        let mut cfg = cfg_with(2.0, 1);
        cfg.protect.kill_all_paused_after_ticks = 3;
        let t = Throttle::new(cfg, 2.0, 1.0, 16.0).expect("invariant");
        let p1 = add_running_pid(&t, "p1", 101).await;
        let p2 = add_running_pid(&t, "p2", 102).await;
        {
            let mut st = t.state.lock().await;
            st.leases.get_mut(&p1).unwrap().state = LeaseState::Paused;
            st.leases.get_mut(&p2).unwrap().state = LeaseState::Paused;
        }
        // No running leases, all paused → select_victim picks paused
        // each tick, killing one per tick. After enough ticks at sub-
        // kill_floor, the last-resort branch kicks in.
        // tick1: kill_floor_run=1, victim=p1 (paused), kills p1.
        // tick2: kill_floor_run=2, victim=p2 (paused), kills p2.
        // (no paused leases left, no more kills)
        let a1 = t.tick(0.5, 0.0, NO_RSS).await;
        let a2 = t.tick(0.5, 0.0, NO_RSS).await;
        assert!(matches!(a1, TickAction::KilledVictim { .. }), "got {a1:?}");
        assert!(matches!(a2, TickAction::KilledVictim { .. }), "got {a2:?}");
        // The Vec<Killed> behaviour requires multiple PAUSED leases at
        // the moment the last-resort threshold trips. Re-set state
        // manually to exercise that path.
        let p3 = add_running_pid(&t, "p3", 103).await;
        let p4 = add_running_pid(&t, "p4", 104).await;
        {
            let mut st = t.state.lock().await;
            st.leases.get_mut(&p3).unwrap().state = LeaseState::Paused;
            st.leases.get_mut(&p4).unwrap().state = LeaseState::Paused;
            // Push kill_floor_run to the brink so the next tick trips.
            st.kill_floor_run = 2; // next inc → 3 == kill_all_after.
        }
        let a3 = t.tick(0.5, 0.0, NO_RSS).await;
        match a3 {
            TickAction::KilledAllPaused(ids) => {
                assert!(
                    ids.contains(&p3) && ids.contains(&p4),
                    "expected p3 and p4 killed, got {ids:?}"
                );
            }
            other => panic!("expected KilledAllPaused, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn classify_oversize_when_victim_exceeds_headroom() {
        // total=16, kill_floor=1.0 → headroom_budget = 15. A 16 GB
        // single-lease RSS exceeds it.
        let c = classify_kill(16.0, 15.0, 1);
        assert_eq!(c, KillClassification::Oversize);
    }

    #[tokio::test]
    async fn classify_external_when_solo_lease_small_rss() {
        // total=16, kill_floor=1.0 → headroom=15. 100 MB victim, solo
        // lease → External (system pressure from non-cap procs).
        let c = classify_kill(0.1, 15.0, 1);
        assert_eq!(c, KillClassification::External);
    }

    #[tokio::test]
    async fn classify_competition_with_multiple_leases() {
        // Multiple leases competing, victim is mid-sized.
        let c = classify_kill(4.0, 15.0, 3);
        assert_eq!(c, KillClassification::Competition);
    }

    #[tokio::test]
    async fn killing_state_excluded_from_victim_selection() {
        // Use grace=10s so the Killing lease is genuinely mid-grace and
        // the escalator doesn't fire ahead of victim selection.
        let mut cfg = cfg_with(2.0, 1);
        cfg.protect.kill_grace_secs = 10;
        let t = Throttle::new(cfg, 2.0, 1.0, 16.0).expect("invariant");
        let killing = add_running_pid(&t, "killing", 100).await;
        let normal = add_running_pid(&t, "normal", 200).await;
        {
            let mut st = t.state.lock().await;
            // Simulate Slice 4 transition: killing is mid-grace, started just now.
            let now = tokio::time::Instant::now();
            let l = st.leases.get_mut(&killing).unwrap();
            l.state = LeaseState::Killing;
            l.kill_started_at = Some(now);
        }
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| match pid {
            100 => Some(10 * 1024 * 1024 * 1024), // biggest, but Killing
            200 => Some(1 * 1024 * 1024 * 1024),
            _ => None,
        };
        // With grace=10s, the per-tick victim path runs and picks
        // `normal`. SIGTERM is sent; we see TermedVictim (no kill yet).
        match t.tick(0.5, 0.0, rss).await {
            TickAction::TermedVictim { id, .. } => assert_eq!(
                id, normal,
                "Killing-state lease must be excluded; expected `normal` victim"
            ),
            other => panic!("expected TermedVictim, got {other:?}"),
        }
    }

    // ── Slice 3: KillEnvelope generation + strategy hints ────────────

    #[tokio::test]
    async fn release_returns_envelope_after_kill() {
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let a = t
            .register(1, "cargo".into(), "cargo test".into(), String::new())
            .await;
        t.attach_pid(a, 999).await;
        let _ = t.tick(0.5, 0.0, NO_RSS).await; // forces kill (External, solo)
        let env = t
            .release(a, None)
            .await
            .kill_envelope
            .expect("envelope must be set after kill");
        assert_eq!(env.classification, KillClassification::External);
        assert_eq!(env.victim_label, "cargo test");
        assert!(
            env.human_message.contains("cargo test"),
            "human_message must mention victim label"
        );
        assert!(
            env.human_message.contains("kill_floor"),
            "human_message must mention kill_floor for context"
        );
    }

    #[tokio::test]
    async fn envelope_action_for_oversize_is_change_strategy() {
        // total=16, kill_floor=1.0, headroom=15. Victim RSS=16 GB → Oversize.
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let a = t
            .register(1, "cargo".into(), "cargo build -j16".into(), String::new())
            .await;
        t.attach_pid(a, 700).await;
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| {
            if pid == 700 {
                Some(16 * 1024 * 1024 * 1024)
            } else {
                None
            }
        };
        let _ = t.tick(0.5, 0.0, rss).await;
        let env = t
            .release(a, None)
            .await
            .kill_envelope
            .expect("envelope expected");
        assert_eq!(env.classification, KillClassification::Oversize);
        match env.action {
            Action::ChangeStrategy { hint, next_step } => {
                assert!(
                    hint.contains("--jobs"),
                    "cargo hint must mention --jobs, got {hint}"
                );
                assert!(
                    next_step.contains("--jobs"),
                    "next_step must echo the hint, got {next_step}"
                );
            }
            other => panic!("expected ChangeStrategy, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn envelope_action_for_competition_is_wait_and_retry() {
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let a = t
            .register(1, "cargo".into(), "cargo a".into(), String::new())
            .await;
        t.attach_pid(a, 800).await;
        let b = t
            .register(1, "cargo".into(), "cargo b".into(), String::new())
            .await;
        t.attach_pid(b, 801).await;
        // Both running, biggest is a (8 GB RSS, fits in headroom=15)
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| match pid {
            800 => Some(8 * 1024 * 1024 * 1024),
            801 => Some(2 * 1024 * 1024 * 1024),
            _ => None,
        };
        let _ = t.tick(0.5, 0.0, rss).await;
        let env = t
            .release(a, None)
            .await
            .kill_envelope
            .expect("envelope expected for killed lease");
        assert_eq!(env.classification, KillClassification::Competition);
        assert!(matches!(env.action, Action::WaitAndRetry { .. }));
        // Sibling brief must include b.
        assert!(
            env.other_leases.iter().any(|l| l.lease == b),
            "sibling brief must list lease b"
        );
    }

    #[test]
    fn strategy_hint_matrix_covers_known_programs() {
        assert!(strategy_hint("cargo").contains("--jobs"));
        assert!(strategy_hint("pytest").contains("-k"));
        assert!(strategy_hint("npm").contains("max-parallel"));
        assert!(strategy_hint("yarn").contains("max-parallel"));
        assert!(strategy_hint("webpack").contains("entry"));
        assert!(strategy_hint("tsc").contains("incremental"));
        assert!(strategy_hint("go").contains("parallel"));
        assert!(strategy_hint("unknown_thing").contains("parallelism"));
        // Path-prefixed program name must still match basename.
        assert!(strategy_hint("/usr/local/bin/cargo").contains("--jobs"));
    }

    #[test]
    fn envelope_json_round_trip() {
        let env = KillEnvelope {
            classification: KillClassification::Competition,
            action: Action::WaitAndRetry {
                suggested_secs: 30,
                next_step: "cap wait && cargo test".into(),
            },
            victim_label: "cargo test".into(),
            victim_rss_gb: 6.5,
            free_gb: 0.8,
            kill_floor_gb: 1.0,
            total_gb: 16.0,
            other_leases: vec![LeaseBrief {
                lease: 7,
                label: "pytest -q".into(),
                state: LeaseState::Running,
                rss_gb: 2.3,
            }],
            human_message: "cap: killed `cargo test`\n".into(),
        };
        let s = serde_json::to_string(&env).expect("serialize");
        let back: KillEnvelope = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(back.classification, env.classification);
        assert_eq!(back.victim_label, env.victim_label);
        assert_eq!(back.other_leases.len(), 1);
        assert_eq!(back.other_leases[0].lease, 7);
        match back.action {
            Action::WaitAndRetry { suggested_secs, .. } => {
                assert_eq!(suggested_secs, 30);
            }
            other => panic!("action variant lost: {other:?}"),
        }
    }

    // ── Slice 4: grace-period kill (SIGTERM → wait → SIGKILL) ─────

    #[tokio::test(start_paused = true)]
    async fn grace_period_sigterm_then_escalate() {
        // Throttle with kill_grace_secs=3 (the default-style production
        // value). tokio::time::pause is on so we can advance virtual
        // time and watch the escalator without sleeping for real.
        let mut cfg = cfg_with(2.0, 1);
        cfg.protect.kill_grace_secs = 3;
        let t = Throttle::new(cfg, 2.0, 1.0, 16.0).expect("invariant");
        let a = t
            .register(1, "cargo".into(), "cargo test".into(), String::new())
            .await;
        t.attach_pid(a, i32::MAX).await;

        // Tick 1: free < kill_floor → SIGTERM, state becomes Killing.
        match t.tick(0.5, 0.0, NO_RSS).await {
            TickAction::TermedVictim { id, .. } => assert_eq!(id, a),
            other => panic!("expected TermedVictim, got {other:?}"),
        }
        {
            let st = t.state.lock().await;
            let l = st.leases.get(&a).unwrap();
            assert_eq!(l.state, LeaseState::Killing);
            assert!(
                l.kill_started_at.is_some(),
                "kill_started_at must be recorded on TermedVictim"
            );
        }

        // Still mid-grace: another tick should NOT escalate.
        tokio::time::advance(std::time::Duration::from_secs(1)).await;
        match t.tick(0.5, 0.0, NO_RSS).await {
            TickAction::Idle | TickAction::TermedVictim { .. } => {
                // Either is acceptable — Idle if no new victim, or
                // TermedVictim if select_victim picked some other lease.
                // What matters: Killing-state `a` was NOT escalated yet.
            }
            TickAction::EscalatedToKill { .. } => {
                panic!("escalated too early — grace was only 1s of 3s")
            }
            other => panic!("unexpected action mid-grace: {other:?}"),
        }
        {
            let st = t.state.lock().await;
            assert_eq!(
                st.leases.get(&a).unwrap().state,
                LeaseState::Killing,
                "still in Killing while grace not yet elapsed"
            );
        }

        // Push past grace: next tick should escalate.
        tokio::time::advance(std::time::Duration::from_secs(5)).await;
        match t.tick(0.5, 0.0, NO_RSS).await {
            TickAction::EscalatedToKill { id } => assert_eq!(id, a),
            other => panic!("expected EscalatedToKill, got {other:?}"),
        }
        {
            let st = t.state.lock().await;
            assert_eq!(
                st.leases.get(&a).unwrap().state,
                LeaseState::Killed,
                "state must be Killed after escalation"
            );
        }
    }

    #[tokio::test]
    async fn grace_zero_skips_sigterm_and_kills_immediately() {
        // grace=0 is the configured "skip SIGTERM" path. cfg_with sets
        // it to 0 already, so this is the cfg_with default behavior.
        let t = Throttle::new(cfg_with(2.0, 1), 2.0, 1.0, 16.0).expect("invariant");
        let a = add_running(&t, "a").await;
        match t.tick(0.5, 0.0, NO_RSS).await {
            TickAction::KilledVictim { id, .. } => assert_eq!(id, a),
            other => panic!("expected immediate KilledVictim, got {other:?}"),
        }
        let st = t.state.lock().await;
        assert_eq!(
            st.leases.get(&a).unwrap().state,
            LeaseState::Killed,
            "no grace = direct Killed, no Killing intermediate state"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn escalator_runs_even_when_pressure_clears() {
        // Failure mode this guards against: SIGTERM'd lease left in
        // Killing forever because free recovered before grace expired
        // and the kill-zone branch never ran again.
        let mut cfg = cfg_with(2.0, 1);
        cfg.protect.kill_grace_secs = 2;
        let t = Throttle::new(cfg, 2.0, 1.0, 16.0).expect("invariant");
        let a = t
            .register(1, "cargo".into(), "cargo test".into(), String::new())
            .await;
        t.attach_pid(a, i32::MAX).await;
        // Tick 1: kill zone → SIGTERM.
        let _ = t.tick(0.5, 0.0, NO_RSS).await;
        // Pressure recovers to "above pause_floor". Without the
        // escalator-runs-first rule, this would leave `a` in Killing
        // forever. Advance past grace and tick at high-free.
        tokio::time::advance(std::time::Duration::from_secs(5)).await;
        match t.tick(8.0, 0.0, NO_RSS).await {
            TickAction::EscalatedToKill { id } => assert_eq!(id, a),
            other => panic!("escalator must fire even at high free, got {other:?}"),
        }
    }

    // ── Slice 5: cap wait + Acquire backpressure + CPU pause ──────

    #[tokio::test]
    async fn is_under_pause_pressure_truth_table() {
        // pause_floor=2.0, pause_load_percent default=80 → load_pause_floor=0.8.
        let t = throttle_with(2.0, 1);
        // mem ok, cpu ok
        assert!(!t.is_under_pause_pressure(8.0, 0.5));
        // mem bad, cpu ok
        assert!(t.is_under_pause_pressure(1.5, 0.5));
        // mem ok, cpu bad
        assert!(t.is_under_pause_pressure(8.0, 1.2));
        // mem bad, cpu bad
        assert!(t.is_under_pause_pressure(1.5, 1.2));
    }

    #[tokio::test]
    async fn cpu_pressure_alone_pauses_newest() {
        // Memory fine, but load_per_core > pause floor → must SIGSTOP.
        // pause_load_percent default = 80 → threshold = 0.8.
        let t = throttle_with(2.0, 1);
        let _a = add_running(&t, "a").await;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let b = add_running(&t, "b").await;
        match t.tick(8.0, 1.5, NO_RSS).await {
            TickAction::PausedNewest(id) => assert_eq!(id, b),
            other => panic!("expected PausedNewest({b}) under CPU pressure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn cpu_pause_disabled_when_load_pause_percent_zero() {
        // pause_load_percent=0 means "never pause for CPU" — even a
        // pegged load must not trigger a SIGSTOP.
        let mut cfg = cfg_with(2.0, 1);
        cfg.protect.pause_load_percent = 0;
        let t = Throttle::new(cfg, 2.0, 1.0, 16.0).expect("invariant");
        let _ = add_running(&t, "a").await;
        assert!(matches!(t.tick(8.0, 100.0, NO_RSS).await, TickAction::Idle));
    }

    #[tokio::test]
    async fn wait_for_capacity_returns_immediately_when_clear() {
        let t = throttle_with(2.0, 1);
        // No pressure has ever been recorded → fast path returns true.
        assert!(
            t.wait_for_capacity(Some(std::time::Duration::from_secs(1)))
                .await
        );
    }

    #[tokio::test(start_paused = true)]
    async fn wait_for_capacity_times_out_when_still_under_pressure() {
        let t = throttle_with(2.0, 1);
        // Drive a tick at sub-pause-floor to set was_under_pressure=true.
        let _ = add_running(&t, "a").await;
        let _ = t.tick(1.5, 0.0, NO_RSS).await;
        // Timeout is small; nothing will clear the pressure.
        let ok = t
            .wait_for_capacity(Some(std::time::Duration::from_millis(200)))
            .await;
        assert!(!ok, "wait must time out, not return ok");
    }

    #[tokio::test]
    async fn wait_for_capacity_wakes_on_headroom_recovery() {
        let t = throttle_with(2.0, 1);
        let _ = add_running(&t, "a").await;
        // First tick: under pressure → was_under_pressure := true.
        let _ = t.tick(1.5, 0.0, NO_RSS).await;
        // Concurrently park a waiter and trigger a recovery tick.
        let tc = t.clone();
        let waiter = tokio::spawn(async move {
            tc.wait_for_capacity(Some(std::time::Duration::from_secs(5)))
                .await
        });
        // Give the waiter a chance to actually park on the Notify.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        // Recovery tick: free above pause_floor, no CPU pressure →
        // edge fires notify_waiters().
        let _ = t.tick(8.0, 0.0, NO_RSS).await;
        let ok = waiter.await.expect("waiter task completed");
        assert!(ok, "waiter must wake on headroom recovery, not time out");
    }

    // ── Run-log record bookkeeping ───────────────────────────────

    #[tokio::test]
    async fn release_record_captures_queue_duration_and_peak_rss() {
        let t = throttle_with(2.0, 1);
        let id = t
            .register(
                7,
                "cargo".into(),
                "cargo test -p cap".into(),
                "/tmp/proj".into(),
            )
            .await;
        // Queue time: the command waits before the client reports its PID.
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        t.attach_pid(id, 4242).await;
        // A healthy-headroom tick records peak RSS + free-at-start.
        let rss: &(dyn Fn(i32) -> Option<u64> + Send + Sync) = &|pid| {
            if pid == 4242 {
                Some(3 * 1024 * 1024 * 1024)
            } else {
                None
            }
        };
        let _ = t.tick(8.0, 0.0, rss).await;
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let rec = t
            .release(id, Some(0))
            .await
            .record
            .expect("a spawned command yields a run record");
        assert_eq!(rec.command, "cargo test -p cap");
        assert_eq!(rec.cwd, "/tmp/proj");
        assert_eq!(rec.exit_code, Some(0));
        assert_eq!(rec.outcome, "completed");
        assert!(rec.kill_classification.is_none());
        assert!(
            rec.queue_ms >= 15,
            "queue_ms reflects pre-spawn wait, got {}",
            rec.queue_ms
        );
        assert!(
            rec.duration_ms >= 15,
            "duration_ms reflects run time, got {}",
            rec.duration_ms
        );
        assert!(
            (rec.peak_rss_gb - 3.0).abs() < 0.05,
            "peak_rss_gb ~3.0, got {}",
            rec.peak_rss_gb
        );
        assert_eq!(rec.free_gb_at_start, Some(8.0));
    }

    #[tokio::test]
    async fn release_record_is_none_when_never_spawned() {
        // A lease that registered but never reached Spawned (client died
        // during Acquire backpressure) didn't actually run → not logged.
        let t = throttle_with(2.0, 1);
        let id = t.register(7, "ls".into(), "ls".into(), String::new()).await;
        let out = t.release(id, None).await;
        assert!(
            out.record.is_none(),
            "never-spawned command must not be logged"
        );
    }

    #[tokio::test]
    async fn snapshot_reports_both_floors_and_load() {
        let t = throttle_with(2.0, 1);
        let snap = t.snapshot(1.5, 0.42).await;
        assert!((snap.pause_floor_gb - 2.0).abs() < 0.01);
        assert!(snap.kill_floor_gb < snap.pause_floor_gb);
        assert!((snap.load_per_core - 0.42).abs() < 0.01);
        // Default pause_load_percent = 80 → 0.8.
        assert!((snap.load_pause_floor - 0.8).abs() < 0.01);
    }
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/throttle.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#rust_source.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-throttle-rs-source-replay-superseded>"
```
