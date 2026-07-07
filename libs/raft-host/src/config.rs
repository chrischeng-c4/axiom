//! [`RaftHost`](crate::RaftHost) tuning.

use std::time::Duration;

/// When the host captures a state-machine snapshot and compacts the raft log.
#[derive(Clone, Copy, Debug)]
pub enum SnapshotPolicy {
    /// Never compact (the log grows; fine for log-broker state machines with no
    /// meaningful snapshot, e.g. relay).
    Disabled,
    /// Compact when `applied_index - snapshot_index >= n`.
    EveryEntries(u64),
    /// The host never auto-compacts; the consumer drives it (e.g. lumen's
    /// periodic RDB snapshotter calls `snapshot_and_compact`).
    External,
}

/// Host timing + snapshot policy.
#[derive(Clone, Copy, Debug)]
pub struct HostConfig {
    /// Logical tick (election/heartbeat clock).
    pub tick: Duration,
    /// Fast outbox pump (ships replies-driven work under the election timeout).
    pub pump: Duration,
    /// Peer RPC timeout.
    pub rpc_timeout: Duration,
    /// How long `propose` waits for its entry to apply before erroring.
    pub propose_timeout: Duration,
    /// Auto-compaction policy.
    pub snapshot: SnapshotPolicy,
}

impl Default for HostConfig {
    fn default() -> Self {
        HostConfig {
            tick: Duration::from_millis(20),
            pump: Duration::from_millis(5),
            rpc_timeout: Duration::from_millis(400),
            propose_timeout: Duration::from_secs(10),
            snapshot: SnapshotPolicy::Disabled,
        }
    }
}
