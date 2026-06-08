//! IPC protocol between cap clients and the cap daemon.
//!
//! Wire format: newline-delimited JSON over a Unix domain socket.
//!
//! Moved into `cap-core` so any tool (the cap CLI, vat, …) can speak it
//! without re-declaring the wire types.

use serde::{Deserialize, Serialize};

pub type LeaseId = u64;

/// Client → Daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Request {
    /// Register intent to run a command. Daemon assigns a lease id
    /// and returns immediately — no admission gate up-front. Live
    /// throttling kicks in once the child PID is known.
    Acquire(AcquireRequest),
    /// Client has spawned the child; daemon now knows what to
    /// SIGSTOP / SIGCONT / SIGKILL when pressure changes.
    Spawned { lease: LeaseId, child_pid: i32 },
    /// Child has exited.
    Release {
        lease: LeaseId,
        exit_code: Option<i32>,
    },
    /// `cap status` — snapshot of leases + pressure.
    Status,
    /// `cap ps` — alias of status.
    Ps,
    /// `cap daemon stop` — graceful shutdown.
    Shutdown,
    /// `cap ping` — health probe.
    Ping,
    /// `cap wait [--timeout N]` — block until system headroom recovers
    /// above both the memory pause floor AND the CPU load pause floor.
    /// The daemon caps wait time at 5 min server-side regardless of
    /// what the client sent. `timeout_secs = None` lets the server cap
    /// apply; `Some(0)` is treated as "non-blocking probe".
    WaitForCapacity { timeout_secs: Option<u64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireRequest {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub label: Option<String>,
    pub client_pid: i32,
}

/// Daemon → Client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Response {
    /// Lease assigned. Client may now spawn the child.
    Lease { lease: LeaseId, nice: i32 },
    /// Spawned acknowledged.
    SpawnedAck,
    /// Release acknowledged. `kill_envelope` is set iff the daemon
    /// SIGKILLed the child due to memory pressure. The envelope is a
    /// structured replacement for the legacy single-string reason —
    /// agents read `classification` + `action` to decide whether to
    /// wait-and-retry or change strategy, and `human_message` for the
    /// rendered diagnostic.
    Released {
        lease: LeaseId,
        kill_envelope: Option<KillEnvelope>,
    },
    /// Snapshot reply.
    Status(StatusSnapshot),
    /// Daemon will exit imminently.
    ShuttingDown,
    /// Health probe reply.
    Pong { version: String },
    /// `WaitForCapacity` resolved: headroom is currently OK on both
    /// memory and CPU axes.
    CapacityOk,
    /// `WaitForCapacity` timed out (client deadline or server hard cap)
    /// before headroom recovered. The CLI surfaces this as exit 124
    /// (matching GNU `timeout`).
    CapacityTimeout,
    /// Error path.
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusSnapshot {
    pub free_mem_gb: f64,
    /// Memory pause floor (free-GB). Below this triggers SIGSTOP.
    pub pause_floor_gb: f64,
    /// Memory kill floor (free-GB). Below this triggers SIGKILL.
    pub kill_floor_gb: f64,
    /// Loadavg(1m) divided by nproc. > load_pause_floor triggers
    /// CPU-side SIGSTOP. 0.0 until Slice 5 wires LoadSampler.
    pub load_per_core: f64,
    /// Threshold for `load_per_core` that triggers pause.
    pub load_pause_floor: f64,
    /// Legacy field kept for older `cap status` clients. Mirrors
    /// `pause_floor_gb`; new code should read `pause_floor_gb`.
    pub min_free_gb: f64,
    pub running: u32,
    pub paused: u32,
    pub leases: Vec<LeaseSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseState {
    /// Registered, waiting for client to send Spawned.
    Pending,
    /// Child is running.
    Running,
    /// Child is SIGSTOPped by the throttler.
    Paused,
    /// Throttler has sent SIGTERM and is waiting `kill_grace_secs`
    /// before escalating to SIGKILL. Excluded from victim re-selection.
    Killing,
    /// Child was SIGKILLed by the throttler.
    Killed,
}

/// Why a particular kill happened — surfaced to the agent so it can
/// decide whether to wait-and-retry, change strategy, or wait for
/// external pressure to clear. Computed at the tick site where lease
/// count and victim RSS are both known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KillClassification {
    /// Multiple cap-managed leases were running and the largest one
    /// was evicted to make room for the others. Retry after `cap wait`.
    Competition,
    /// A single lease's RSS by itself exceeded the system's headroom
    /// budget. Retrying the same command will hit the same wall —
    /// agent must change strategy (lower `--jobs`, split the task).
    Oversize,
    /// Only one (or zero) cap lease was running, but free memory was
    /// still below the kill floor. Non-cap processes (LSPs, browsers,
    /// IDEs) are eating the budget. Agent should `cap wait` and let
    /// the system recover; retrying immediately will fail again.
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseSnapshot {
    pub lease: LeaseId,
    pub client_pid: i32,
    pub child_pid: Option<i32>,
    pub label: String,
    pub state: LeaseState,
    pub age_secs: u64,
}

/// One-line summary of another lease that was active at the moment a
/// kill happened. Useful for the agent to understand whether it was
/// competing with its own siblings or with non-cap work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseBrief {
    pub lease: LeaseId,
    pub label: String,
    pub state: LeaseState,
    pub rss_gb: f64,
}

/// Actionable suggestion attached to a kill, derived from the
/// classification. Phrased as a hint — agents may choose to apply it,
/// fall back to their own strategy, or escalate to a human.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Action {
    /// Competition with other cap leases. `cap wait` for capacity,
    /// then retry the same command — the winner finishes and frees RAM.
    WaitAndRetry {
        suggested_secs: u64,
        next_step: String,
    },
    /// Single lease alone was over the headroom budget. Retrying as-is
    /// will hit the same wall — agent must change strategy (lower
    /// parallelism, split the workload, etc).
    ChangeStrategy { hint: String, next_step: String },
    /// Non-cap processes are eating the budget. Agent should pause,
    /// optionally inspect what's hogging RAM, then `cap wait`.
    InspectAndWait {
        suggested_secs: u64,
        next_step: String,
    },
}

/// Structured kill report — superseded the old `killed_reason: String`.
/// Surfaced to clients on `Response::Released` and rendered as a
/// multi-line stderr message via `human_message`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillEnvelope {
    pub classification: KillClassification,
    pub action: Action,
    pub victim_label: String,
    pub victim_rss_gb: f64,
    pub free_gb: f64,
    pub kill_floor_gb: f64,
    pub total_gb: f64,
    pub other_leases: Vec<LeaseBrief>,
    /// Pre-formatted multi-line message for `eprintln!` on the client.
    /// Agents that want structure should read the typed fields instead.
    pub human_message: String,
}
