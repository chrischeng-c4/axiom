// CODEGEN-BEGIN
//! The `RaftStateMachine` a consumer supplies to [`crate::RaftHost`].

use std::any::Any;
use std::io::{Read, Write};

use raft_core::Index;

/// Opaque committed-entry bytes (raft_core's `RaftEntry.command`). The host never
/// looks inside — the state machine encodes/decodes its own commands.
pub type Command = Vec<u8>;

/// Opaque application-owned memory admission retained by the host from Raft
/// index allocation through the matching state-machine apply callback.
pub type AdmissionPermit = Box<dyn Any + Send>;

/// Resources prepared without excluding ordered apply. This owned handle may
/// retain a product's checkpoint-writer reservation across the capture cut.
pub trait SnapshotPreparation: Send + 'static {
    /// Freeze exactly `index` while the host serializes state-machine changes.
    /// This step must not perform file I/O or wait for memory or a save permit.
    /// A consumer that cannot represent this exact prefix must return an error.
    fn capture_at(self: Box<Self>, index: Index) -> anyhow::Result<Box<dyn PreparedSnapshot>>;
}

/// An immutable captured state whose output no longer needs the host's apply
/// serialization lease. Dropping the handle must release its owned resources.
pub trait PreparedSnapshot: Send + 'static {
    /// Encode the captured prefix outside the host's state-machine lease.
    /// Failure prevents compaction; it must not be reported as a complete cut.
    fn write_to(self: Box<Self>, writer: &mut dyn Write) -> anyhow::Result<()>;
}

/// The consumer's replicated state machine. The host owns the **only** applier:
/// every committed entry is fed to [`apply`](RaftStateMachine::apply) exactly
/// once after success, in index order, on every node, from one ordered worker.
/// File reads, capacity waits, and callbacks run outside the node mutex.
/// [`snapshot`](RaftStateMachine::snapshot) / [`restore`](RaftStateMachine::restore)
/// bound the log (compaction) and let a lagging/fresh replica catch up.
///
/// Implementors are `&self` interior-mutable (engines are `Arc<_>` with internal
/// locks); the host holds an `Arc<dyn RaftStateMachine>`.
pub trait RaftStateMachine: Send + Sync + 'static {
    /// Admit one leader-side proposal before the host allocates a Raft index.
    ///
    /// A returned permit becomes host-owned once an index is allocated. The
    /// default preserves the existing unadmitted behavior for all consumers.
    fn admit_proposal(&self, _command: &[u8]) -> anyhow::Result<Option<AdmissionPermit>> {
        Ok(None)
    }

    /// Apply one committed command while retaining the exact permit that was
    /// created before its leader-side index allocation. The default retains the
    /// permit through the synchronous `apply` callback and then drops it. For
    /// compatibility, an `apply` error is treated as a completed domain no-op
    /// only when the callback has already advanced its public applied floor to
    /// `index`. An error before that floor remains incomplete.
    fn apply_admitted(
        &self,
        index: Index,
        command: &[u8],
        permit: Option<AdmissionPermit>,
    ) -> anyhow::Result<()> {
        let _permit = permit;
        match self.apply(index, command) {
            Ok(()) => Ok(()),
            Err(error) if self.applied_index() >= index => {
                tracing::warn!(
                    index,
                    error = %error,
                    "state machine reported an error after advancing its applied floor; preserving legacy completed no-op"
                );
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Wire capability required before the host sends a coordinated snapshot
    /// to every voter. `None` keeps the legacy behavior. A versioned product
    /// returns a stable token so a new leader cannot compact through a voter
    /// that still runs an older snapshot decoder during a rolling upgrade.
    fn snapshot_capability(&self) -> Option<&'static str> {
        None
    }

    /// Apply one committed command at `index` (1-based, strictly increasing, once
    /// per entry). `index` equals the raft log index (for lumen, the WAL seq).
    /// Return `Ok(())` only after advancing the durable applied watermark.
    /// A normal domain refusal should be a completed no-op represented in the
    /// consumer's own outcome store. For legacy callers of the default
    /// `apply_admitted` adapter, an `Err` after this public floor reaches
    /// `index` is also treated as a completed no-op. An `Err` before that floor
    /// is incomplete: the host stops and retains this committed head for
    /// recovery before any later command. An `apply_admitted` override controls
    /// its own error handling.
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()>;

    /// Serialize the full state as of the last applied index. The host ships
    /// these bytes via `InstallSnapshot` and stores them through `node.compact`.
    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()>;

    /// Prepare an optional immutable snapshot path outside apply serialization.
    /// File validation and waits for checkpoint capacity belong here. The host
    /// then calls `capture_at` under its lease and exports after releasing it.
    /// `None` preserves `snapshot_at` and its existing serialization behavior.
    fn preflight_snapshot(&self) -> anyhow::Result<Option<Box<dyn SnapshotPreparation>>> {
        Ok(None)
    }

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
