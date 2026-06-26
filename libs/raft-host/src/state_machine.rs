//! The `RaftStateMachine` a consumer supplies to [`crate::RaftHost`].

use raftcore::Index;

/// Opaque committed-entry bytes (raftcore's `RaftEntry.command`). The host never
/// looks inside — the state machine encodes/decodes its own commands.
pub type Command = Vec<u8>;

/// The consumer's replicated state machine. The host owns the **only** applier:
/// every committed entry is fed to [`apply`](RaftStateMachine::apply) exactly
/// once, in index order, on every node, from a single task under the node lock.
/// [`snapshot`](RaftStateMachine::snapshot) / [`restore`](RaftStateMachine::restore)
/// bound the log (compaction) and let a lagging/fresh replica catch up.
///
/// Implementors are `&self` interior-mutable (engines are `Arc<_>` with internal
/// locks); the host holds an `Arc<dyn RaftStateMachine>`.
pub trait RaftStateMachine: Send + Sync + 'static {
    /// Apply one committed command at `index` (1-based, strictly increasing, once
    /// per entry). `index` equals the raft log index (for lumen, the WAL seq).
    /// An `Err` is logged by the host and the entry is treated as applied
    /// (no-op) so the log keeps advancing — the implementor must still advance
    /// its own [`applied_index`](RaftStateMachine::applied_index) past `index`.
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()>;

    /// Serialize the full state as of the last applied index. The host ships
    /// these bytes via `InstallSnapshot` and stores them through `node.compact`.
    fn snapshot(&self) -> anyhow::Result<Vec<u8>>;

    /// Replace the entire state from snapshot bytes (a follower installing a
    /// leader's snapshot, or cold-start). After this, [`applied_index`] must
    /// return the snapshot's index.
    fn restore(&self, snapshot: &[u8]) -> anyhow::Result<()>;

    /// Highest index durably applied by this state machine (survives restart).
    /// Drives the host's commit-wait (read-your-write) and the idempotency floor.
    fn applied_index(&self) -> Index;
}
