// HANDWRITE-BEGIN gap="sift-framed-journal-state-machine" tracker="1605" reason="Implement CRC-framed event journal snapshot/restore and the RaftStateMachine adapter."
//! Shared durability and Raft state-machine adapter for Sift's raw journal.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use anyhow::Result;
use raft_host::{Index, RaftStateMachine};
use serde::{Deserialize, Serialize};

use crate::{DurableJournal, IncomingEvent, StoredEvent};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct JournalSnapshot {
    pub applied_index: u64,
    pub events: Vec<StoredEvent>,
}

impl JournalSnapshot {
    pub(crate) fn from_events(events: Vec<StoredEvent>) -> Self {
        let applied_index = events.last().map(|event| event.cursor).unwrap_or(0);
        Self {
            applied_index,
            events,
        }
    }
}

/// The only replicated applier for Sift events. RaftHost calls this adapter in
/// committed-index order, so an acknowledged replica write has passed through
/// the same durable journal boundary on every voter.
pub struct SiftStateMachine {
    journal: Arc<DurableJournal>,
    applied_index: AtomicU64,
}

impl SiftStateMachine {
    pub fn new(journal: Arc<DurableJournal>) -> Self {
        let applied_index = journal.last_cursor();
        Self {
            journal,
            applied_index: AtomicU64::new(applied_index),
        }
    }
}

impl RaftStateMachine for SiftStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        let event: IncomingEvent = serde_json::from_slice(command)?;
        self.journal.append_at(index, event.into_inner())?;
        self.applied_index.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&JournalSnapshot::from_events(
            self.journal.snapshot_events(),
        ))
        .map_err(Into::into)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snapshot: JournalSnapshot = serde_json::from_slice(snapshot)?;
        self.journal.restore_snapshot(snapshot.events)?;
        self.applied_index
            .store(snapshot.applied_index, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied_index.load(Ordering::Acquire)
    }
}

// HANDWRITE-END
