// CODEGEN-BEGIN
//! The `RaftStateMachine` a consumer supplies to [`crate::RaftHost`].

use std::io::{Read, Write};

use raft_core::Index;

/// Opaque committed-entry bytes (raft_core's `RaftEntry.command`). The host never
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
    /// Wire capability required before the host sends a coordinated snapshot
    /// to every voter. `None` keeps the legacy behavior. A versioned product
    /// returns a stable token so a new leader cannot compact through a voter
    /// that still runs an older snapshot decoder during a rolling upgrade.
    fn snapshot_capability(&self) -> Option<&'static str> {
        None
    }

    /// Apply one committed command at `index` (1-based, strictly increasing, once
    /// per entry). `index` equals the raft log index (for lumen, the WAL seq).
    /// An `Err` is logged by the host and the entry is treated as applied
    /// (no-op) so the log keeps advancing — the implementor must still advance
    /// its own [`applied_index`](RaftStateMachine::applied_index) past `index`.
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()>;

    /// Serialize the full state as of the last applied index. The host ships
    /// these bytes via `InstallSnapshot` and stores them through `node.compact`.
    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()>;

    /// Serialize a state that is safe for compaction through `index`.
    ///
    /// Most state machines only support the current applied head. They inherit
    /// this default, which refuses an older prefix instead of attaching the
    /// wrong state to a Raft snapshot index. Durable log-backed products can
    /// override this hook and return a checkpoint backed by their own storage.
    fn snapshot_at(&self, index: Index, writer: &mut dyn Write) -> anyhow::Result<()> {
        let applied = self.applied_index();
        if index != applied {
            anyhow::bail!(
                "state machine cannot snapshot Raft prefix {index}; current applied index is {applied}"
            );
        }
        self.snapshot(writer)
    }

    /// Validate snapshot bytes without changing durable product state.
    ///
    /// The host runs this check before it publishes the new Raft snapshot.
    /// Implementors with a strict snapshot format should override it. The
    /// default keeps existing consumers source-compatible; a later restore
    /// failure still latches the host as failed.
    fn validate_snapshot(&self, _reader: &mut dyn Read) -> anyhow::Result<()> {
        Ok(())
    }

    /// Replace the entire state from snapshot bytes (a follower installing a
    /// leader's snapshot, or cold-start). After this, [`applied_index`](RaftStateMachine::applied_index) must
    /// return the snapshot's index.
    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()>;

    /// Highest index durably applied by this state machine (survives restart).
    /// Drives the host's commit-wait (read-your-write) and the idempotency floor.
    fn applied_index(&self) -> Index;
}
// CODEGEN-END
