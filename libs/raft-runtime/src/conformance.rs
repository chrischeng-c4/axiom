//! Deterministic, in-process host for adversarial Raft recovery tests.
//!
//! This module deliberately has no runtime, socket, timer, or wall-clock
//! dependency.  A harness drives logical time with [`DeterministicHost::tick`]
//! and moves opaque envelopes itself.  It uses the production host's cold-start,
//! apply, persistence, and peer-lane primitives so its result is evidence about
//! the shipped driver rather than a second implementation of it.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use raft_core::{
    DemotionRefused, Index, Membership, NodeId, PromotionRefused, RaftMsg, RaftNode,
    RemovalRefused, Role,
};
use sha2::{Digest, Sha256};

use crate::config::SnapshotPolicy;
use crate::host::{apply_ready, cold_start, persist_node, PeerLaneQueue, SNAPSHOT_CHUNK_SIZE};
use crate::{RaftStateMachine, RaftStore};

/// Stable on-disk trace schema for adversarial-recovery replay files.
pub const TRACE_SCHEMA: &str = "raft-runtime/adversarial-recovery/v1";

/// The kind of one message scheduled by a deterministic host.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvelopeKind {
    Vote,
    VoteResponse,
    Append,
    AppendResponse,
    InstallSnapshot,
    InstallSnapshotResponse,
    TimeoutNow,
}

/// Stable metadata for a pending envelope.  The actual Raft wire message stays
/// opaque so a test can schedule it but cannot accidentally forge a different
/// message under the same identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvelopeMeta {
    pub id: u64,
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EnvelopeKind,
    pub fingerprint: String,
}

/// An opaque, cloneable outbound message.  It may only be delivered with
/// [`DeterministicHost::receive`].
#[derive(Clone, Debug)]
pub struct PendingEnvelope {
    meta: EnvelopeMeta,
    message: RaftMsg,
}

impl PendingEnvelope {
    pub fn meta(&self) -> &EnvelopeMeta {
        &self.meta
    }
}

/// A command supplied to the deterministic state-machine host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateMachineOperation {
    Command(Vec<u8>),
}

impl From<Vec<u8>> for StateMachineOperation {
    fn from(command: Vec<u8>) -> Self {
        Self::Command(command)
    }
}

/// A small, serializable-in-spirit observation of one node.  It intentionally
/// excludes volatile peer replication details so traces stay stable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeView {
    pub id: NodeId,
    pub role: ConformanceRole,
    pub term: u64,
    pub leader: Option<NodeId>,
    pub commit_index: Index,
    pub last_index: Index,
    pub snapshot_index: Index,
    pub resident_log_entries: usize,
    pub membership: Membership,
    pub joint: bool,
}

/// Deterministic role spelling.  This avoids exposing raft-core's internal
/// role type while still making safety assertions readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConformanceRole {
    Follower,
    Candidate,
    Leader,
}

/// Membership operation failures that a conformance trace may expect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConformanceMembershipError {
    NotLeader,
    AddLearnerRefused,
    Promote(PromotionRefused),
    Demote(DemotionRefused),
    Remove(RemovalRefused),
}

impl fmt::Display for ConformanceMembershipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ConformanceMembershipError {}

/// Errors returned by deterministic host actions.  Store errors carry only a
/// stable operation and kind; the original `io::Error` is deliberately not
/// retained because it is platform text, not conformance state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StepError {
    Store {
        operation: &'static str,
        kind: std::io::ErrorKind,
    },
    StateMachine {
        operation: &'static str,
        message: String,
    },
    WrongRecipient {
        expected: NodeId,
        actual: NodeId,
    },
    NotLeader,
    Membership(ConformanceMembershipError),
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for StepError {}

/// One deterministic Raft host.  Drop it to model a crash, then call `open`
/// with the same [`RaftStore`] and a new state machine to model restart.
pub struct DeterministicHost {
    id: NodeId,
    node: RaftNode,
    store: RaftStore,
    sm: Arc<dyn RaftStateMachine>,
    lanes: HashMap<NodeId, PeerLaneQueue>,
    envelope_epoch: u32,
    next_envelope_id: u32,
}

impl DeterministicHost {
    /// Open a host from an empty or durable store.  Unlike the historical
    /// production `spawn_inner`, this test host surfaces a corrupt-store load.
    pub fn open(
        id: NodeId,
        membership: Membership,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
    ) -> Result<Self, StepError> {
        Self::open_with_envelope_epoch(id, membership, store, sm, id as u32)
    }

    /// Open with a trace-owned epoch.  Assign a new epoch when a trace drops
    /// and reopens a host so envelope ids remain unique and replayable.
    pub fn open_with_envelope_epoch(
        id: NodeId,
        membership: Membership,
        store: RaftStore,
        sm: Arc<dyn RaftStateMachine>,
        envelope_epoch: u32,
    ) -> Result<Self, StepError> {
        let mut node = match store.load().map_err(|e| StepError::Store {
            operation: "load",
            kind: e.kind(),
        })? {
            Some(state) => RaftNode::from_persisted(id, &membership, state),
            None => RaftNode::new(id, &membership),
        };
        cold_start(&mut node, sm.as_ref(), true).map_err(|e| StepError::StateMachine {
            operation: "cold-start",
            message: e.to_string(),
        })?;
        let mut host = Self {
            id,
            node,
            store,
            sm,
            lanes: HashMap::new(),
            envelope_epoch,
            next_envelope_id: 1,
        };
        host.pump()?;
        Ok(host)
    }

    /// Advance exactly one logical Raft tick.
    pub fn tick(&mut self) -> Result<(), StepError> {
        self.node.tick();
        self.settle()
    }

    /// Move the node outbox into per-peer FIFO lanes.  Only consecutive,
    /// unsent Append messages coalesce because this uses `PeerLaneQueue`.
    pub fn pump(&mut self) -> Result<(), StepError> {
        for outgoing in self.node.take_outgoing() {
            self.lanes
                .entry(outgoing.to)
                .or_default()
                .enqueue(outgoing.msg);
        }
        self.persist()
    }

    /// Peer ids that have at least one deliverable envelope, in stable order.
    pub fn ready_peers(&self) -> Vec<NodeId> {
        let mut peers: Vec<_> = self
            .lanes
            .iter()
            .filter_map(|(&peer, lane)| (!lane.is_empty()).then_some(peer))
            .collect();
        peers.sort_unstable();
        peers
    }

    /// Exact number of envelopes still queued in the shared peer lanes.
    pub fn pending_len(&self) -> usize {
        self.lanes.values().map(PeerLaneQueue::len).sum()
    }

    /// Remove one pending envelope for `peer` from that peer's FIFO lane.
    pub fn take_next(&mut self, peer: NodeId) -> Option<PendingEnvelope> {
        let message = self.lanes.get_mut(&peer)?.dequeue()?;
        let meta = EnvelopeMeta {
            id: ((self.envelope_epoch as u64) << 32) | self.next_envelope_id as u64,
            from: self.id,
            to: peer,
            kind: envelope_kind(&message),
            fingerprint: fingerprint(&message),
        };
        self.next_envelope_id = self
            .next_envelope_id
            .checked_add(1)
            .expect("trace envelope id space exhausted");
        Some(PendingEnvelope { meta, message })
    }

    /// Deliver one opaque envelope to this host.
    pub fn receive(&mut self, envelope: PendingEnvelope) -> Result<(), StepError> {
        if envelope.meta.to != self.id {
            return Err(StepError::WrongRecipient {
                expected: self.id,
                actual: envelope.meta.to,
            });
        }
        self.node.handle(envelope.meta.from, envelope.message);
        self.settle()
    }

    pub fn try_propose(&mut self, operation: StateMachineOperation) -> Result<Index, StepError> {
        let StateMachineOperation::Command(command) = operation;
        let index = self.node.propose(command).ok_or(StepError::NotLeader)?;
        self.settle()?;
        Ok(index)
    }

    pub fn try_add_learner(&mut self, peer: NodeId) -> Result<Index, StepError> {
        if !self.node.is_leader() {
            return Err(StepError::NotLeader);
        }
        let index = self.node.add_learner(peer).ok_or(StepError::Membership(
            ConformanceMembershipError::AddLearnerRefused,
        ))?;
        self.settle()?;
        Ok(index)
    }

    pub fn promote_learner(&mut self, peer: NodeId) -> Result<Index, StepError> {
        if !self.node.is_leader() {
            return Err(StepError::NotLeader);
        }
        let index = self
            .node
            .promote_learner(peer)
            .map_err(|e| StepError::Membership(ConformanceMembershipError::Promote(e)))?;
        self.settle()?;
        Ok(index)
    }

    pub fn demote_voter(&mut self, peer: NodeId) -> Result<Index, StepError> {
        if !self.node.is_leader() {
            return Err(StepError::NotLeader);
        }
        let index = self
            .node
            .demote_voter(peer)
            .map_err(|e| StepError::Membership(ConformanceMembershipError::Demote(e)))?;
        self.settle()?;
        Ok(index)
    }

    pub fn remove_member(&mut self, peer: NodeId) -> Result<Index, StepError> {
        if !self.node.is_leader() {
            return Err(StepError::NotLeader);
        }
        let index = self
            .node
            .remove_member(peer)
            .map_err(|e| StepError::Membership(ConformanceMembershipError::Remove(e)))?;
        self.settle()?;
        Ok(index)
    }

    /// Capture the state-machine snapshot and compact all applied entries.
    pub fn snapshot_and_compact(&mut self) -> Result<Index, StepError> {
        let applied = self.sm.applied_index();
        if applied <= self.node.snapshot_index() {
            return Ok(self.node.snapshot_index());
        }
        let mut sink = crate::ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
        self.sm
            .snapshot(&mut sink)
            .map_err(|e| StepError::StateMachine {
                operation: "snapshot",
                message: e.to_string(),
            })?;
        self.node.compact(applied, sink.into_bytes());
        self.settle()?;
        Ok(applied)
    }

    pub fn view(&self) -> NodeView {
        NodeView {
            id: self.id,
            role: match self.node.role() {
                Role::Follower => ConformanceRole::Follower,
                Role::Candidate => ConformanceRole::Candidate,
                Role::Leader => ConformanceRole::Leader,
            },
            term: self.node.current_term(),
            leader: self.node.leader(),
            commit_index: self.node.commit_index(),
            last_index: self.node.last_index(),
            snapshot_index: self.node.snapshot_index(),
            resident_log_entries: self.node.log_len(),
            membership: self.node.conf_state().membership.clone(),
            joint: self.node.is_joint(),
        }
    }

    pub fn store(&self) -> &RaftStore {
        &self.store
    }

    fn settle(&mut self) -> Result<(), StepError> {
        self.persist()?;
        apply_ready(
            &mut self.node,
            self.sm.as_ref(),
            None,
            SnapshotPolicy::Disabled,
            true,
        )
        .map_err(|e| StepError::StateMachine {
            operation: "apply-ready",
            message: e.to_string(),
        })?;
        // Applying a configuration entry can append the leave-joint entry.
        // Persist it before exposing a new envelope or returning to the trace.
        self.pump()
    }

    fn persist(&self) -> Result<(), StepError> {
        persist_node(&self.store, &self.node).map_err(|e| StepError::Store {
            operation: "save",
            kind: e.kind(),
        })
    }
}

fn envelope_kind(message: &RaftMsg) -> EnvelopeKind {
    match message {
        RaftMsg::Vote(_) => EnvelopeKind::Vote,
        RaftMsg::VoteResp(_) => EnvelopeKind::VoteResponse,
        RaftMsg::Append(_) => EnvelopeKind::Append,
        RaftMsg::AppendResp(_) => EnvelopeKind::AppendResponse,
        RaftMsg::InstallSnapshot(_) => EnvelopeKind::InstallSnapshot,
        RaftMsg::InstallSnapshotResp(_) => EnvelopeKind::InstallSnapshotResponse,
        RaftMsg::TimeoutNow(_) => EnvelopeKind::TimeoutNow,
    }
}

fn fingerprint(message: &RaftMsg) -> String {
    let payload = match message {
        RaftMsg::Vote(v) => serde_json::to_vec(v),
        RaftMsg::VoteResp(v) => serde_json::to_vec(v),
        RaftMsg::Append(v) => serde_json::to_vec(v),
        RaftMsg::AppendResp(v) => serde_json::to_vec(v),
        RaftMsg::InstallSnapshot(v) => serde_json::to_vec(v),
        RaftMsg::InstallSnapshotResp(v) => serde_json::to_vec(v),
        RaftMsg::TimeoutNow(v) => serde_json::to_vec(v),
    }
    .expect("raft wire messages are serializable");
    let mut hasher = Sha256::new();
    hasher.update([envelope_kind(message) as u8]);
    hasher.update(payload);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicU64, Ordering};
    use tempfile::TempDir;

    struct CountingSm(AtomicU64);
    impl RaftStateMachine for CountingSm {
        fn apply(&self, index: Index, _: &[u8]) -> anyhow::Result<()> {
            self.0.store(index, Ordering::Release);
            Ok(())
        }
        fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
            writer.write_all(&self.0.load(Ordering::Acquire).to_le_bytes())?;
            Ok(())
        }
        fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
            let mut bytes = [0; 8];
            reader.read_exact(&mut bytes)?;
            self.0.store(u64::from_le_bytes(bytes), Ordering::Release);
            Ok(())
        }
        fn applied_index(&self) -> Index {
            self.0.load(Ordering::Acquire)
        }
    }

    #[test]
    fn single_voter_is_deterministic_and_persists_before_reopen() {
        let dir = TempDir::new().unwrap();
        let store =
            RaftStore::open(dir.path().to_str().unwrap(), 0, crate::FsyncPolicy::Always).unwrap();
        let sm = Arc::new(CountingSm(AtomicU64::new(0)));
        let mut host = DeterministicHost::open(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            store,
            sm.clone(),
        )
        .unwrap();
        for _ in 0..50 {
            host.tick().unwrap();
        }
        assert_eq!(host.view().role, ConformanceRole::Leader);
        assert_eq!(host.try_propose(vec![7].into()).unwrap(), 1);
        let store =
            RaftStore::open(dir.path().to_str().unwrap(), 0, crate::FsyncPolicy::Always).unwrap();
        drop(host);
        let after = Arc::new(CountingSm(AtomicU64::new(0)));
        let reopened = DeterministicHost::open(
            0,
            Membership {
                voters: vec![9],
                learners: vec![],
            },
            store,
            after.clone(),
        )
        .unwrap();
        assert_eq!(after.applied_index(), 1);
        assert_eq!(reopened.view().membership.voters, vec![0]);
    }

    #[test]
    fn nonleaders_report_not_leader_for_all_admission_paths() {
        let dir = TempDir::new().unwrap();
        let store =
            RaftStore::open(dir.path().to_str().unwrap(), 1, crate::FsyncPolicy::Always).unwrap();
        let sm = Arc::new(CountingSm(AtomicU64::new(0)));
        let mut host = DeterministicHost::open(
            1,
            Membership {
                voters: vec![0, 1, 2],
                learners: vec![],
            },
            store,
            sm,
        )
        .unwrap();
        assert!(matches!(
            host.try_propose(vec![1].into()),
            Err(StepError::NotLeader)
        ));
        assert!(matches!(host.try_add_learner(3), Err(StepError::NotLeader)));
        assert!(matches!(host.promote_learner(3), Err(StepError::NotLeader)));
        assert!(matches!(host.demote_voter(0), Err(StepError::NotLeader)));
        assert!(matches!(host.remove_member(0), Err(StepError::NotLeader)));
    }
}
