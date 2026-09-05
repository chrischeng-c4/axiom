// CODEGEN-BEGIN
//! Self-contained Raft consensus core (no external dependency).
//!
//! [`RaftNode`] is **step-driven**: it never spawns timers or threads. A driver
//! calls [`tick`](RaftNode::tick) to advance logical time and
//! [`handle`](RaftNode::handle) to feed it an incoming [`RaftMsg`]; the node
//! accumulates replies/heartbeats in an outbox drained via
//! [`take_outgoing`](RaftNode::take_outgoing). This makes the whole protocol a
//! deterministic state machine that a test can simulate exactly (no real
//! network / clock), and a production driver can wrap with an h2c transport.
//!
//! Replicated-state-machine model: the Raft log holds opaque **command** bytes.
//! Once an entry commits (acked by a majority of voters), every node surfaces it
//! via [`take_committed`](RaftNode::take_committed) for the consumer to apply to
//! its own state machine.
//!
//! **Snapshots / log compaction** keep the log bounded for large state machines:
//! a consumer that has applied up to some index snapshots its state machine and
//! calls [`compact`](RaftNode::compact); the Raft log before that index is
//! dropped. A leader replicating to a follower whose next index has been
//! compacted away ships the snapshot (`InstallSnapshot`) instead of replaying
//! the whole history; the follower installs it and surfaces the bytes via
//! [`take_installed_snapshot`](RaftNode::take_installed_snapshot).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Stable node identity (in k8s, the StatefulSet ordinal).
pub type NodeId = u64;
pub type Term = u64;
/// 1-based Raft log index; 0 means "before the first entry".
pub type Index = u64;

/// Logical ticks before a voter starts an election (distinct per node so the
/// deterministic simulation does not livelock on split votes).
// With raft-runtime's 20ms production tick this is a one-second minimum.
// Stateful adopters persist proposals before releasing the node lock; a
// 200ms election window made ordinary bursts of durable fsyncs look like a
// failed leader and caused avoidable term churn.
pub const ELECTION_TIMEOUT_FLOOR_TICKS: u64 = 50;
/// Ticks between leader heartbeats / replication pushes.
pub const HEARTBEAT_INTERVAL_TICKS: u64 = 3;

/// Discriminator distinguishing client commands from internal configuration entries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    #[default]
    Command,
    Config,
}

/// One replicated command entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaftEntry {
    pub term: Term,
    pub index: Index,
    pub command: Vec<u8>,
    #[serde(default)]
    pub kind: EntryKind,
}

/// The durable hard state of a Raft node: what must survive a restart so the
/// node never double-votes in a term or forgets acknowledged entries. Carries
/// the compaction point + snapshot bytes so a restarted node can still serve
/// lagging followers.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedState {
    pub term: Term,
    pub voted_for: Option<NodeId>,
    pub log: Vec<RaftEntry>,
    /// Highest index known committed when the hard state was saved. Persisting
    /// this lets a cold host replay committed resident entries before serving
    /// reads, rather than waiting for a new-term proposal to re-establish the
    /// commit watermark.
    #[serde(default)]
    pub commit_index: Index,
    #[serde(default)]
    pub snapshot_index: Index,
    #[serde(default)]
    pub snapshot_term: Term,
    #[serde(default)]
    pub snapshot: Vec<u8>,
    #[serde(default)]
    pub conf: Option<ConfState>,
}

/// Borrowed durable state used by runtimes that must persist a large log
/// without cloning it on every heartbeat or proposal.
pub struct PersistedStateRef<'a> {
    pub term: Term,
    pub voted_for: Option<NodeId>,
    pub log: &'a [RaftEntry],
    pub commit_index: Index,
    pub snapshot_index: Index,
    pub snapshot_term: Term,
    pub snapshot: &'a [u8],
    pub conf: Option<&'a ConfState>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteReq {
    pub term: Term,
    pub candidate: NodeId,
    pub last_log_index: Index,
    pub last_log_term: Term,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteResp {
    pub term: Term,
    pub granted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendReq {
    pub term: Term,
    pub leader: NodeId,
    pub prev_log_index: Index,
    pub prev_log_term: Term,
    pub entries: Vec<RaftEntry>,
    pub leader_commit: Index,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendResp {
    pub term: Term,
    pub success: bool,
    /// Highest log index the follower now matches the leader on.
    pub match_index: Index,
}

/// Ship a state-machine snapshot to a follower whose needed entries have been
/// compacted away. `data` is opaque (the consumer's serialized state machine).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallSnapshotReq {
    pub term: Term,
    pub leader: NodeId,
    pub snapshot_index: Index,
    pub snapshot_term: Term,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallSnapshotResp {
    pub term: Term,
    /// True only when this voter accepted the requested snapshot identity, or
    /// already holds a snapshot that supersedes it. Older peers omit this
    /// field and therefore fail closed.
    #[serde(default)]
    pub accepted: bool,
    /// The snapshot index the follower now holds.
    pub snapshot_index: Index,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeoutNowReq {
    pub term: Term,
    pub leader: NodeId,
}

#[derive(Clone, Debug)]
pub enum RaftMsg {
    Vote(VoteReq),
    VoteResp(VoteResp),
    Append(AppendReq),
    AppendResp(AppendResp),
    InstallSnapshot(InstallSnapshotReq),
    InstallSnapshotResp(InstallSnapshotResp),
    TimeoutNow(TimeoutNowReq),
}

/// A message the driver must deliver to node `to`.
#[derive(Clone, Debug)]
pub struct Outgoing {
    pub to: NodeId,
    pub msg: RaftMsg,
}

/// How a driver delivers a node's outgoing messages. The production driver
/// implements this over h2c; tests use an in-process bus.
pub trait RaftTransport {
    fn deliver(&mut self, from: NodeId, out: Outgoing);
}

/// Why a leader refused a promotion request (#3570).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionRefused {
    NotLeader,
    NotCaughtUp { matched: Index, target: Index },
    TransitionInFlight,
}

/// Why a leader refused a leadership transfer request (#3571).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferRefused {
    NotLeader,
    NotAVoter {
        target: NodeId,
    },
    NotCaughtUp {
        target: NodeId,
        matched: Index,
        last_index: Index,
    },
}

/// Why a leader refused a demotion request (#3572).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemotionRefused {
    NotLeader,
    IsTheLeader { target: NodeId },
    TransitionInFlight,
    NotAVoter { target: NodeId },
    ToleranceWouldDrop { before: usize, after: usize },
    WouldEmptyVoterSet { target: NodeId },
}

/// Why a leader refused a removal request (#3572).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemovalRefused {
    NotLeader,
    TransitionInFlight,
    NotAMember { target: NodeId },
    IsTheLeader { target: NodeId },
    ToleranceWouldDrop { before: usize, after: usize },
    WouldEmptyVoterSet { target: NodeId },
}

/// Cluster membership for one Raft group.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Membership {
    pub voters: Vec<NodeId>,
    pub learners: Vec<NodeId>,
}

/// Monotonically sequenced cluster membership.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfState {
    pub membership: Membership,
    #[serde(default)]
    pub outgoing: Option<Vec<NodeId>>,
    pub generation: u64,
}

impl ConfState {
    pub fn encode(&self) -> Vec<u8> {
        let outgoing_len = self.outgoing.as_ref().map(|o| o.len()).unwrap_or(0);
        let mut buf = Vec::with_capacity(
            24 + (self.membership.voters.len() + self.membership.learners.len() + outgoing_len) * 8
                + 8,
        );
        buf.extend_from_slice(&self.generation.to_le_bytes());
        buf.extend_from_slice(&(self.membership.voters.len() as u64).to_le_bytes());
        for &v in &self.membership.voters {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        buf.extend_from_slice(&(self.membership.learners.len() as u64).to_le_bytes());
        for &l in &self.membership.learners {
            buf.extend_from_slice(&l.to_le_bytes());
        }
        match &self.outgoing {
            Some(outgoing) => {
                buf.extend_from_slice(&(outgoing.len() as u64).to_le_bytes());
                for &v in outgoing {
                    buf.extend_from_slice(&v.to_le_bytes());
                }
            }
            None => {
                buf.extend_from_slice(&0u64.to_le_bytes());
            }
        }
        buf
    }

    pub fn decode_with_len(bytes: &[u8]) -> Option<(ConfState, usize)> {
        if bytes.len() < 24 {
            return None;
        }
        let mut offset = 0;
        let generation = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;
        let voters_len = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?) as usize;
        offset += 8;
        if voters_len > (bytes.len() - offset - 8) / 8 {
            return None;
        }
        let mut voters = Vec::with_capacity(voters_len);
        for _ in 0..voters_len {
            voters.push(u64::from_le_bytes(
                bytes[offset..offset + 8].try_into().ok()?,
            ));
            offset += 8;
        }
        let learners_len = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?) as usize;
        offset += 8;
        if learners_len > (bytes.len() - offset) / 8 {
            return None;
        }
        let mut learners = Vec::with_capacity(learners_len);
        for _ in 0..learners_len {
            learners.push(u64::from_le_bytes(
                bytes[offset..offset + 8].try_into().ok()?,
            ));
            offset += 8;
        }
        let outgoing = if offset == bytes.len() {
            None
        } else {
            if bytes.len() - offset < 8 {
                return None;
            }
            let outgoing_len =
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?) as usize;
            offset += 8;
            if outgoing_len == 0 {
                None
            } else {
                if outgoing_len > (bytes.len() - offset) / 8 {
                    return None;
                }
                let mut outgoing_voters = Vec::with_capacity(outgoing_len);
                for _ in 0..outgoing_len {
                    outgoing_voters.push(u64::from_le_bytes(
                        bytes[offset..offset + 8].try_into().ok()?,
                    ));
                    offset += 8;
                }
                Some(outgoing_voters)
            }
        };
        Some((
            ConfState {
                membership: Membership { voters, learners },
                outgoing,
                generation,
            },
            offset,
        ))
    }

    pub fn decode(bytes: &[u8]) -> Option<ConfState> {
        Self::decode_with_len(bytes).map(|(conf, _)| conf)
    }
}

/// Derive membership for node ids `0..n`: voters are the largest **odd** prefix
/// (`n` if odd else `n-1`), the trailing even node becomes a non-voting learner.
/// So the voter count is always odd (1,1,3,3,5,5,…) → clean majorities, and
/// every extra even node is a read-only learner. `n == 0` is treated as 1.
pub fn auto_membership(n: u64) -> Membership {
    let n = n.max(1);
    let voters = if n % 2 == 1 { n } else { n - 1 };
    Membership {
        voters: (0..voters).collect(),
        learners: (voters..n).collect(),
    }
}

/// A single Raft-group participant.
pub struct RaftNode {
    id: NodeId,
    peers: Vec<NodeId>, // all other members (voters + learners)
    is_voter: bool,
    conf_state: ConfState,

    role: Role,
    current_term: Term,
    voted_for: Option<NodeId>,
    /// In-memory log; `log[0]` has index `snapshot_index + 1`.
    log: Vec<RaftEntry>,
    resident_log_bytes: usize,
    commit_index: Index,
    last_applied: Index,

    // compaction
    snapshot_index: Index,
    snapshot_term: Term,
    snapshot: Vec<u8>,
    installed_snapshot: Option<Vec<u8>>,

    // leader-only, per peer
    next_index: HashMap<NodeId, Index>,
    match_index: HashMap<NodeId, Index>,
    learner_read_targets: HashMap<NodeId, Index>,

    // election
    votes: HashSet<NodeId>,
    election_elapsed: u64,
    election_timeout: u64,
    heartbeat_elapsed: u64,
    /// Last known leader for this term (drives producer redirect-to-leader).
    leader_id: Option<NodeId>,

    // leadership transfer
    transfer_in_flight: Option<NodeId>,
    transfer_elapsed: u64,

    outbox: Vec<Outgoing>,
}

impl RaftNode {
    /// Create a node `id` within `membership` (starts as Follower at term 0).
    pub fn new(id: NodeId, membership: &Membership) -> RaftNode {
        let mut members: Vec<NodeId> = membership
            .voters
            .iter()
            .chain(membership.learners.iter())
            .copied()
            .collect();
        members.sort_unstable();
        let peers = members.into_iter().filter(|m| *m != id).collect();
        let mut learner_read_targets = HashMap::new();
        for &l in &membership.learners {
            learner_read_targets.insert(l, 0);
        }
        RaftNode {
            id,
            peers,
            is_voter: membership.voters.contains(&id),
            conf_state: ConfState {
                membership: membership.clone(),
                outgoing: None,
                generation: 0,
            },
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            resident_log_bytes: 0,
            commit_index: 0,
            last_applied: 0,
            snapshot_index: 0,
            snapshot_term: 0,
            snapshot: Vec::new(),
            installed_snapshot: None,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            learner_read_targets,
            votes: HashSet::new(),
            election_elapsed: 0,
            // distinct per node so one voter always times out first.
            election_timeout: ELECTION_TIMEOUT_FLOOR_TICKS + id,
            heartbeat_elapsed: 0,
            leader_id: None,
            transfer_in_flight: None,
            transfer_elapsed: 0,
            outbox: Vec::new(),
        }
    }

    /// Restore a node from durable [`PersistedState`]: term, votedFor, log and
    /// the compaction point are recovered; volatile state (role, commit/apply)
    /// restarts as a Follower at the snapshot point and is re-derived via
    /// replication. Committed entries re-apply idempotently downstream.
    pub fn from_persisted(id: NodeId, membership: &Membership, state: PersistedState) -> RaftNode {
        let conf = state.conf.unwrap_or_else(|| ConfState {
            membership: membership.clone(),
            outgoing: None,
            generation: 0,
        });
        let mut node = RaftNode::new(id, &conf.membership);
        node.conf_state = conf;
        node.current_term = state.term;
        node.voted_for = state.voted_for;
        node.log = state.log;
        node.resident_log_bytes = node.log.iter().map(|entry| entry.command.len()).sum();
        node.snapshot_index = state.snapshot_index;
        node.snapshot_term = state.snapshot_term;
        node.snapshot = state.snapshot;
        node.commit_index = state
            .commit_index
            .max(node.snapshot_index)
            .min(node.last_index());
        node.last_applied = node.snapshot_index;
        if node.snapshot_index > 0 && !node.snapshot.is_empty() {
            node.installed_snapshot = Some(node.snapshot.clone());
        }
        node
    }

    /// Snapshot the durable hard state for the consumer's store.
    pub fn persisted(&self) -> PersistedState {
        PersistedState {
            term: self.current_term,
            voted_for: self.voted_for,
            log: self.log.clone(),
            commit_index: self.commit_index,
            snapshot_index: self.snapshot_index,
            snapshot_term: self.snapshot_term,
            snapshot: self.snapshot.clone(),
            conf: Some(self.conf_state.clone()),
        }
    }

    /// Borrow the durable hard state without copying the resident log or
    /// snapshot bytes.
    pub fn persisted_ref(&self) -> PersistedStateRef<'_> {
        PersistedStateRef {
            term: self.current_term,
            voted_for: self.voted_for,
            log: &self.log,
            commit_index: self.commit_index,
            snapshot_index: self.snapshot_index,
            snapshot_term: self.snapshot_term,
            snapshot: &self.snapshot,
            conf: Some(&self.conf_state),
        }
    }

    pub fn conf_state(&self) -> &ConfState {
        &self.conf_state
    }

    /// Whether this node has a joint configuration in force.
    pub fn is_joint(&self) -> bool {
        self.conf_state.outgoing.is_some()
    }

    /// Return an eligible caught-up voter to receive leadership, or `None` if
    /// this node is not the leader, is the only voter, or no other voter has
    /// replicated this node's whole log (#3664).
    pub fn handoff_candidate(&self) -> Option<NodeId> {
        if self.role != Role::Leader {
            return None;
        }
        let last_index = self.last_index();
        self.conf_state
            .membership
            .voters
            .iter()
            .copied()
            .filter(|&id| id != self.id)
            .find(|&id| {
                let matched = self.match_index.get(&id).copied().unwrap_or(0);
                matched >= last_index
            })
    }

    /// Transfer leadership to a named caught-up voter (#3571).
    pub fn transfer_leadership(&mut self, target: NodeId) -> Result<(), TransferRefused> {
        if self.role != Role::Leader {
            return Err(TransferRefused::NotLeader);
        }
        if !self.conf_state.membership.voters.contains(&target) {
            return Err(TransferRefused::NotAVoter { target });
        }
        let matched = if target == self.id {
            self.last_index()
        } else {
            self.match_index.get(&target).copied().unwrap_or(0)
        };
        let last_index = self.last_index();
        if matched < last_index {
            return Err(TransferRefused::NotCaughtUp {
                target,
                matched,
                last_index,
            });
        }
        self.transfer_in_flight = Some(target);
        self.transfer_elapsed = 0;
        self.send(
            target,
            RaftMsg::TimeoutNow(TimeoutNowReq {
                term: self.current_term,
                leader: self.id,
            }),
        );
        Ok(())
    }

    /// Promote a caught-up learner to voter through a joint configuration.
    pub fn promote_learner(&mut self, peer: NodeId) -> Result<Index, PromotionRefused> {
        if self.role != Role::Leader {
            return Err(PromotionRefused::NotLeader);
        }
        if self.is_joint() || self.transfer_in_flight.is_some() {
            return Err(PromotionRefused::TransitionInFlight);
        }
        if self
            .log
            .iter()
            .any(|e| e.index > self.commit_index && e.kind == EntryKind::Config)
        {
            return Err(PromotionRefused::TransitionInFlight);
        }
        let matched = self.learner_matched(peer).unwrap_or(0);
        let target = self.learner_read_target(peer).unwrap_or(0);
        if matched < target {
            return Err(PromotionRefused::NotCaughtUp { matched, target });
        }
        let mut new_voters = self.conf_state.membership.voters.clone();
        if !new_voters.contains(&peer) {
            new_voters.push(peer);
            new_voters.sort_unstable();
        }
        let mut new_learners = self.conf_state.membership.learners.clone();
        new_learners.retain(|l| *l != peer);

        let outgoing = Some(self.conf_state.membership.voters.clone());
        let conf = ConfState {
            membership: Membership {
                voters: new_voters,
                learners: new_learners,
            },
            outgoing,
            generation: self.conf_state.generation + 1,
        };
        let idx = self
            .propose_config(conf)
            // unreachable: propose_config returns None only when self.role != Role::Leader or self.transfer_in_flight.is_some(), which are excluded above by self.role != Role::Leader and self.is_joint() || self.transfer_in_flight.is_some().
            .ok_or(PromotionRefused::TransitionInFlight)?;
        Ok(idx)
    }

    /// Demote a voter to a learner through a joint configuration (#3572).
    pub fn demote_voter(&mut self, peer: NodeId) -> Result<Index, DemotionRefused> {
        if self.role != Role::Leader {
            return Err(DemotionRefused::NotLeader);
        }
        if peer == self.id {
            return Err(DemotionRefused::IsTheLeader { target: peer });
        }
        if self.is_joint() || self.transfer_in_flight.is_some() {
            return Err(DemotionRefused::TransitionInFlight);
        }
        if self
            .log
            .iter()
            .any(|e| e.index > self.commit_index && e.kind == EntryKind::Config)
        {
            return Err(DemotionRefused::TransitionInFlight);
        }
        if !self.conf_state.membership.voters.contains(&peer) {
            return Err(DemotionRefused::NotAVoter { target: peer });
        }
        let mut new_voters = self.conf_state.membership.voters.clone();
        new_voters.retain(|v| *v != peer);
        if new_voters.is_empty() {
            return Err(DemotionRefused::WouldEmptyVoterSet { target: peer });
        }
        let n = self.conf_state.membership.voters.len();
        let before = n.saturating_sub(n / 2 + 1);
        let after = (n - 1).saturating_sub((n - 1) / 2 + 1);
        if after < before {
            return Err(DemotionRefused::ToleranceWouldDrop { before, after });
        }
        let mut new_learners = self.conf_state.membership.learners.clone();
        if !new_learners.contains(&peer) {
            new_learners.push(peer);
            new_learners.sort_unstable();
        }

        let outgoing = Some(self.conf_state.membership.voters.clone());
        let conf = ConfState {
            membership: Membership {
                voters: new_voters,
                learners: new_learners,
            },
            outgoing,
            generation: self.conf_state.generation + 1,
        };
        let idx = self
            .propose_config(conf)
            // unreachable: propose_config returns None only when self.role != Role::Leader or self.transfer_in_flight.is_some(), which are excluded above by self.role != Role::Leader and self.is_joint() || self.transfer_in_flight.is_some().
            .ok_or(DemotionRefused::TransitionInFlight)?;
        Ok(idx)
    }

    /// Remove a member from the group through a joint configuration (#3572).
    pub fn remove_member(&mut self, peer: NodeId) -> Result<Index, RemovalRefused> {
        if self.role != Role::Leader {
            return Err(RemovalRefused::NotLeader);
        }
        if peer == self.id {
            return Err(RemovalRefused::IsTheLeader { target: peer });
        }
        if self.is_joint() || self.transfer_in_flight.is_some() {
            return Err(RemovalRefused::TransitionInFlight);
        }
        if self
            .log
            .iter()
            .any(|e| e.index > self.commit_index && e.kind == EntryKind::Config)
        {
            return Err(RemovalRefused::TransitionInFlight);
        }
        let is_voter = self.conf_state.membership.voters.contains(&peer);
        let is_learner = self.conf_state.membership.learners.contains(&peer);
        if !is_voter && !is_learner {
            return Err(RemovalRefused::NotAMember { target: peer });
        }
        let mut new_voters = self.conf_state.membership.voters.clone();
        new_voters.retain(|v| *v != peer);
        if is_voter {
            if new_voters.is_empty() {
                return Err(RemovalRefused::WouldEmptyVoterSet { target: peer });
            }
            let n = self.conf_state.membership.voters.len();
            let before = n.saturating_sub(n / 2 + 1);
            let after = (n - 1).saturating_sub((n - 1) / 2 + 1);
            if after < before {
                return Err(RemovalRefused::ToleranceWouldDrop { before, after });
            }
        }
        let mut new_learners = self.conf_state.membership.learners.clone();
        new_learners.retain(|l| *l != peer);

        let outgoing = Some(self.conf_state.membership.voters.clone());
        let conf = ConfState {
            membership: Membership {
                voters: new_voters,
                learners: new_learners,
            },
            outgoing,
            generation: self.conf_state.generation + 1,
        };
        let idx = self
            .propose_config(conf)
            // unreachable: propose_config returns None only when self.role != Role::Leader or self.transfer_in_flight.is_some(), which are excluded above by self.role != Role::Leader and self.is_joint() || self.transfer_in_flight.is_some().
            .ok_or(RemovalRefused::TransitionInFlight)?;
        Ok(idx)
    }

    /// Adopt a superseding configuration state. Refuses configurations whose
    /// generation does not strictly exceed the one in force.
    pub fn adopt_conf(&mut self, conf: ConfState) -> bool {
        if conf.generation <= self.conf_state.generation {
            return false;
        }
        let mut members: Vec<NodeId> = conf
            .membership
            .voters
            .iter()
            .chain(conf.membership.learners.iter())
            .chain(conf.outgoing.iter().flatten())
            .copied()
            .collect();
        members.sort_unstable();
        members.dedup();
        self.peers = members.into_iter().filter(|m| *m != self.id).collect();
        self.is_voter = conf.membership.voters.contains(&self.id)
            || conf
                .outgoing
                .as_ref()
                .map_or(false, |o| o.contains(&self.id));
        self.conf_state = conf;
        for l in &self.conf_state.membership.learners {
            self.learner_read_targets
                .entry(*l)
                .or_insert(self.commit_index);
        }
        if self.role == Role::Leader {
            let next = self.last_index() + 1;
            for p in &self.peers {
                self.next_index.entry(*p).or_insert(next);
                self.match_index.entry(*p).or_insert(0);
            }
        }
        true
    }

    pub fn id(&self) -> NodeId {
        self.id
    }
    pub fn role(&self) -> Role {
        self.role
    }
    pub fn is_leader(&self) -> bool {
        self.role == Role::Leader
    }
    pub fn is_voter(&self) -> bool {
        self.is_voter
    }
    pub fn current_term(&self) -> Term {
        self.current_term
    }
    pub fn commit_index(&self) -> Index {
        self.commit_index
    }
    /// Highest log index (covers compacted prefix): `snapshot_index + log.len()`.
    pub fn last_index(&self) -> Index {
        self.snapshot_index + self.log.len() as Index
    }
    /// Last index folded into a snapshot (0 = none).
    pub fn snapshot_index(&self) -> Index {
        self.snapshot_index
    }

    /// Command payload bytes retained in the resident Raft log.
    pub fn resident_log_bytes(&self) -> usize {
        self.resident_log_bytes
    }

    /// Return the term stored at `index` when that prefix is still known.
    ///
    /// Snapshot coordinators use this before they send a prospective snapshot
    /// to peers. `None` means the requested index is outside this node's known
    /// prefix.
    pub fn term_at_index(&self, index: Index) -> Option<Term> {
        if index == 0 || index > self.last_index() {
            return None;
        }
        Some(self.term_at(index))
    }
    /// Number of resident log entries (post-compaction).
    pub fn log_len(&self) -> usize {
        self.log.len()
    }
    /// Last known leader for the current term (for producer redirect).
    pub fn leader(&self) -> Option<NodeId> {
        self.leader_id
    }

    /// Highest index this leader has recorded as replicated to `peer`, or `None`
    /// if this node is not the leader or `peer` is not an admitted learner.
    pub fn learner_matched(&self, peer: NodeId) -> Option<Index> {
        if self.role != Role::Leader || !self.conf_state.membership.learners.contains(&peer) {
            return None;
        }
        self.match_index.get(&peer).copied().or(Some(0))
    }

    /// The index `peer` must replicate to before it is fit to serve reads, or
    /// `None` if `peer` is not an admitted learner.
    pub fn learner_read_target(&self, peer: NodeId) -> Option<Index> {
        if !self.conf_state.membership.learners.contains(&peer) {
            return None;
        }
        self.learner_read_targets.get(&peer).copied()
    }

    /// Whether an admitted learner has caught up to its recorded read target,
    /// or `None` if this node is not the leader or `peer` is not an admitted learner.
    pub fn learner_read_eligible(&self, peer: NodeId) -> Option<bool> {
        let matched = self.learner_matched(peer)?;
        let target = self.learner_read_target(peer)?;
        Some(matched >= target)
    }

    /// Number of committed entries an admitted learner is behind, measured
    /// against this leader's commit index when asked (the live freshness
    /// question, as opposed to [`learner_read_eligible`](Self::learner_read_eligible)
    /// which answers the admission question), or `None` if this node is not the
    /// leader or `peer` is not an admitted learner.
    pub fn learner_replication_gap(&self, peer: NodeId) -> Option<Index> {
        let matched = self.learner_matched(peer)?;
        Some(self.commit_index.saturating_sub(matched))
    }

    fn last_term(&self) -> Term {
        self.log
            .last()
            .map(|e| e.term)
            .unwrap_or(self.snapshot_term)
    }

    /// Term of the entry at `index` (snapshot point or a resident entry).
    fn term_at(&self, index: Index) -> Term {
        if index == 0 {
            0
        } else if index <= self.snapshot_index {
            self.snapshot_term
        } else {
            let pos = (index - self.snapshot_index - 1) as usize;
            self.log.get(pos).map(|e| e.term).unwrap_or(0)
        }
    }

    /// Drain messages the driver must deliver.
    pub fn take_outgoing(&mut self) -> Vec<Outgoing> {
        std::mem::take(&mut self.outbox)
    }

    /// Newly committed entries (in index order); advances `last_applied`.
    /// Configuration entries are adopted into force and withheld from the
    /// consumer.
    pub fn take_committed(&mut self) -> Vec<RaftEntry> {
        let mut out = Vec::new();
        while self.last_applied < self.commit_index {
            let idx = self.last_applied + 1;
            let pos = (idx - self.snapshot_index - 1) as usize;
            let entry = &self.log[pos];
            if entry.kind == EntryKind::Config {
                if let Some(conf) = ConfState::decode(&entry.command) {
                    self.adopt_conf(conf);
                    if self.role == Role::Leader && self.is_joint() {
                        self.check_leave_joint();
                    }
                }
            } else {
                out.push(entry.clone());
            }
            self.last_applied = idx;
        }
        out
    }

    /// A snapshot received from a leader, for the consumer to load into its state
    /// machine (call once after [`handle`] processes an `InstallSnapshot`).
    pub fn take_installed_snapshot(&mut self) -> Option<Vec<u8>> {
        self.installed_snapshot.take()
    }

    /// Reject an incoming snapshot after the consumer could not restore its
    /// state-machine bytes.
    ///
    /// The higher term and leader identity still take effect, but the local
    /// snapshot index and resident log stay unchanged. The response advertises
    /// the old index so the leader retries instead of treating this voter as
    /// caught up.
    pub fn reject_install_snapshot(&mut self, req: InstallSnapshotReq) {
        if req.term < self.current_term {
            let (term, snapshot_index) = (self.current_term, self.snapshot_index);
            self.send(
                req.leader,
                RaftMsg::InstallSnapshotResp(InstallSnapshotResp {
                    term,
                    accepted: false,
                    snapshot_index,
                }),
            );
            return;
        }
        self.step_down(req.term);
        self.leader_id = Some(req.leader);
        let (term, snapshot_index) = (self.current_term, self.snapshot_index);
        self.send(
            req.leader,
            RaftMsg::InstallSnapshotResp(InstallSnapshotResp {
                term,
                accepted: false,
                snapshot_index,
            }),
        );
    }

    /// Compact the log up through `up_to` (must be applied): the consumer has
    /// snapshotted its state machine to `snapshot` bytes, so entries `<= up_to`
    /// can be dropped. The snapshot is what a leader later ships to a follower
    /// whose next index has been compacted away.
    pub fn compact(&mut self, up_to: Index, snapshot: Vec<u8>) {
        if up_to <= self.snapshot_index || up_to > self.last_applied {
            return;
        }
        let term = self.term_at(up_to);
        let drop = (up_to - self.snapshot_index) as usize;
        let drop = drop.min(self.log.len());
        let removed = self.log[..drop]
            .iter()
            .map(|entry| entry.command.len())
            .sum::<usize>();
        self.log.drain(0..drop);
        self.resident_log_bytes = self.resident_log_bytes.saturating_sub(removed);
        self.snapshot_index = up_to;
        self.snapshot_term = term;
        self.snapshot = snapshot;
    }

    fn send(&mut self, to: NodeId, msg: RaftMsg) {
        self.outbox.push(Outgoing { to, msg });
    }

    /// Advance one logical tick: leaders heartbeat, voters may start an election.
    pub fn tick(&mut self) {
        self.election_elapsed += 1;
        self.heartbeat_elapsed += 1;
        if self.role == Role::Leader {
            if self.transfer_in_flight.is_some() {
                self.transfer_elapsed += 1;
                if self.transfer_elapsed >= self.election_timeout {
                    self.transfer_in_flight = None;
                    self.transfer_elapsed = 0;
                }
            }
            if self.heartbeat_elapsed >= HEARTBEAT_INTERVAL_TICKS {
                self.heartbeat_elapsed = 0;
                self.broadcast_append();
            }
        } else if self.is_voter && self.election_elapsed >= self.election_timeout {
            self.start_election();
        }
    }

    fn start_election(&mut self) {
        self.current_term += 1;
        self.role = Role::Candidate;
        self.voted_for = Some(self.id);
        self.leader_id = None;
        self.votes.clear();
        self.votes.insert(self.id);
        self.election_elapsed = 0;
        let (lli, llt) = (self.last_index(), self.last_term());
        let term = self.current_term;
        let mut vote_targets = self.conf_state.membership.voters.clone();
        if let Some(outgoing) = &self.conf_state.outgoing {
            vote_targets.extend(outgoing);
        }
        vote_targets.sort_unstable();
        vote_targets.dedup();
        let peers: Vec<NodeId> = vote_targets.into_iter().filter(|v| *v != self.id).collect();
        for v in peers {
            self.send(
                v,
                RaftMsg::Vote(VoteReq {
                    term,
                    candidate: self.id,
                    last_log_index: lli,
                    last_log_term: llt,
                }),
            );
        }
        // A sole voter wins immediately.
        self.maybe_become_leader();
    }

    fn maybe_become_leader(&mut self) {
        if self.role != Role::Candidate {
            return;
        }
        let incoming_granted = self
            .votes
            .iter()
            .filter(|v| self.conf_state.membership.voters.contains(v))
            .count();
        let incoming_maj = self.conf_state.membership.voters.len() / 2 + 1;
        let outgoing_satisfied = match &self.conf_state.outgoing {
            Some(outgoing) => {
                let outgoing_granted = self.votes.iter().filter(|v| outgoing.contains(v)).count();
                let outgoing_maj = outgoing.len() / 2 + 1;
                outgoing_granted >= outgoing_maj
            }
            None => true,
        };
        if incoming_granted >= incoming_maj && outgoing_satisfied {
            self.become_leader();
        }
    }

    fn become_leader(&mut self) {
        self.role = Role::Leader;
        self.leader_id = Some(self.id);
        let next = self.last_index() + 1;
        self.next_index.clear();
        self.match_index.clear();
        for p in self.peers.clone() {
            self.next_index.insert(p, next);
            self.match_index.insert(p, 0);
        }
        self.heartbeat_elapsed = 0;
        self.transfer_in_flight = None;
        self.transfer_elapsed = 0;
        self.broadcast_append();
        if self.is_joint() {
            self.check_leave_joint();
        }
    }

    fn check_leave_joint(&mut self) {
        if self.role != Role::Leader || !self.is_joint() {
            return;
        }
        let has_pending = self.log.iter().any(|e| {
            e.index > self.commit_index
                && e.kind == EntryKind::Config
                && e.term == self.current_term
        });
        if !has_pending {
            let conf = ConfState {
                membership: self.conf_state.membership.clone(),
                outgoing: None,
                generation: self.conf_state.generation + 1,
            };
            self.propose_config(conf);
        }
    }

    fn step_down(&mut self, term: Term) {
        if term > self.current_term {
            self.current_term = term;
            self.voted_for = None;
        }
        self.role = Role::Follower;
        self.election_elapsed = 0;
        self.transfer_in_flight = None;
        self.transfer_elapsed = 0;
    }

    fn broadcast_append(&mut self) {
        for p in self.peers.clone() {
            self.send_append_to(p);
        }
    }

    fn send_append_to(&mut self, peer: NodeId) {
        let next = *self
            .next_index
            .get(&peer)
            .unwrap_or(&(self.last_index() + 1));
        // Needed entries compacted away → ship the snapshot instead.
        if next <= self.snapshot_index {
            let (term, si, st) = (self.current_term, self.snapshot_index, self.snapshot_term);
            let data = self.snapshot.clone();
            self.send(
                peer,
                RaftMsg::InstallSnapshot(InstallSnapshotReq {
                    term,
                    leader: self.id,
                    snapshot_index: si,
                    snapshot_term: st,
                    data,
                }),
            );
            return;
        }
        let prev_index = next.saturating_sub(1);
        let prev_term = self.term_at(prev_index);
        let entries: Vec<RaftEntry> = self
            .log
            .iter()
            .filter(|e| e.index >= next)
            .cloned()
            .collect();
        let (term, commit) = (self.current_term, self.commit_index);
        self.send(
            peer,
            RaftMsg::Append(AppendReq {
                term,
                leader: self.id,
                prev_log_index: prev_index,
                prev_log_term: prev_term,
                entries,
                leader_commit: commit,
            }),
        );
    }

    /// Append a command on the leader and replicate it. Returns its index, or
    /// `None` if this node is not the leader.
    pub fn propose(&mut self, command: Vec<u8>) -> Option<Index> {
        if self.role != Role::Leader || self.transfer_in_flight.is_some() {
            return None;
        }
        let index = self.last_index() + 1;
        self.resident_log_bytes = self.resident_log_bytes.saturating_add(command.len());
        self.log.push(RaftEntry {
            term: self.current_term,
            index,
            command,
            kind: EntryKind::Command,
        });
        self.broadcast_append();
        self.maybe_commit(); // sole voter commits immediately
        Some(index)
    }

    /// Append a configuration entry on the leader and replicate it. Returns its
    /// index, or `None` if this node is not the leader.
    pub fn propose_config(&mut self, conf: ConfState) -> Option<Index> {
        if self.role != Role::Leader || self.transfer_in_flight.is_some() {
            return None;
        }
        let index = self.last_index() + 1;
        let command = conf.encode();
        self.resident_log_bytes = self.resident_log_bytes.saturating_add(command.len());
        self.log.push(RaftEntry {
            term: self.current_term,
            index,
            command,
            kind: EntryKind::Config,
        });
        self.broadcast_append();
        self.maybe_commit();
        Some(index)
    }

    /// Append a configuration entry adding a learner on the leader and replicate
    /// it. Returns its index, or `None` if this node is not the leader.
    pub fn add_learner(&mut self, peer: NodeId) -> Option<Index> {
        if self.role != Role::Leader || self.transfer_in_flight.is_some() {
            return None;
        }
        let mut conf = self.conf_state.clone();
        conf.generation += 1;
        if !conf.membership.learners.contains(&peer) {
            conf.membership.learners.push(peer);
            conf.membership.learners.sort_unstable();
        }
        self.propose_config(conf)
    }

    /// Feed an incoming message from `from`.
    pub fn handle(&mut self, from: NodeId, msg: RaftMsg) {
        match msg {
            RaftMsg::Vote(req) => self.handle_vote(from, req),
            RaftMsg::VoteResp(resp) => self.handle_vote_resp(from, resp),
            RaftMsg::Append(req) => self.handle_append(req),
            RaftMsg::AppendResp(resp) => self.handle_append_resp(from, resp),
            RaftMsg::InstallSnapshot(req) => self.handle_install_snapshot(req),
            RaftMsg::InstallSnapshotResp(resp) => self.handle_install_snapshot_resp(from, resp),
            RaftMsg::TimeoutNow(req) => self.handle_timeout_now(req),
        }
    }

    fn handle_vote(&mut self, from: NodeId, req: VoteReq) {
        if req.term > self.current_term {
            self.step_down(req.term);
        }
        let up_to_date = req.last_log_term > self.last_term()
            || (req.last_log_term == self.last_term() && req.last_log_index >= self.last_index());
        let grant = req.term == self.current_term
            && (self.voted_for.is_none() || self.voted_for == Some(req.candidate))
            && up_to_date;
        if grant {
            self.voted_for = Some(req.candidate);
            self.election_elapsed = 0;
        }
        let term = self.current_term;
        self.send(
            from,
            RaftMsg::VoteResp(VoteResp {
                term,
                granted: grant,
            }),
        );
    }

    fn handle_vote_resp(&mut self, from: NodeId, resp: VoteResp) {
        if resp.term > self.current_term {
            self.step_down(resp.term);
            return;
        }
        if self.role == Role::Candidate && resp.term == self.current_term && resp.granted {
            self.votes.insert(from);
            self.maybe_become_leader();
        }
    }

    fn handle_append(&mut self, req: AppendReq) {
        let leader = req.leader;
        if req.term < self.current_term {
            let term = self.current_term;
            self.send(
                leader,
                RaftMsg::AppendResp(AppendResp {
                    term,
                    success: false,
                    match_index: 0,
                }),
            );
            return;
        }
        // Valid leader for this (or a newer) term: become its follower.
        self.step_down(req.term);
        self.leader_id = Some(leader);

        // Log matching: the entry preceding the new ones must agree. Anything at
        // or below our snapshot point is implicitly matched.
        if req.prev_log_index > self.last_index()
            || (req.prev_log_index > self.snapshot_index
                && self.term_at(req.prev_log_index) != req.prev_log_term)
        {
            let term = self.current_term;
            // Report the greatest prefix this request can safely assume did
            // not match. The leader uses this both as a fast backoff hint and
            // to discard a failure response that arrived after a newer
            // successful AppendEntries response for the same peer.
            let match_index = req.prev_log_index.saturating_sub(1).min(self.last_index());
            self.send(
                leader,
                RaftMsg::AppendResp(AppendResp {
                    term,
                    success: false,
                    match_index,
                }),
            );
            return;
        }

        // Append, skipping entries already covered by the snapshot and truncating
        // any conflicting suffix.
        for e in &req.entries {
            if e.index <= self.snapshot_index {
                continue;
            }
            let pos = (e.index - self.snapshot_index - 1) as usize;
            if pos < self.log.len() {
                if self.log[pos].term != e.term {
                    let removed = self.log[pos..]
                        .iter()
                        .map(|entry| entry.command.len())
                        .sum::<usize>();
                    self.log.truncate(pos);
                    self.resident_log_bytes = self.resident_log_bytes.saturating_sub(removed);
                    self.resident_log_bytes =
                        self.resident_log_bytes.saturating_add(e.command.len());
                    self.log.push(e.clone());
                }
            } else {
                self.resident_log_bytes = self.resident_log_bytes.saturating_add(e.command.len());
                self.log.push(e.clone());
            }
        }
        let match_index = req.prev_log_index + req.entries.len() as Index;
        if req.leader_commit > self.commit_index {
            self.commit_index = req.leader_commit.min(self.last_index());
        }
        let term = self.current_term;
        self.send(
            leader,
            RaftMsg::AppendResp(AppendResp {
                term,
                success: true,
                match_index,
            }),
        );
    }

    fn handle_append_resp(&mut self, from: NodeId, resp: AppendResp) {
        if resp.term > self.current_term {
            self.step_down(resp.term);
            return;
        }
        if self.role != Role::Leader || resp.term != self.current_term {
            return;
        }
        if resp.success {
            // Multiple h2 requests to one peer can complete out of order.
            // Replication progress is monotonic: a stale success must never
            // move match_index/next_index behind a newer acknowledgement.
            let matched = self.match_index.entry(from).or_insert(0);
            *matched = (*matched).max(resp.match_index);
            let next = self.next_index.entry(from).or_insert(1);
            *next = (*next).max(matched.saturating_add(1));
            let old = self.commit_index;
            self.maybe_commit();
            if self.commit_index > old {
                // Propagate the new commit to everyone.
                self.broadcast_append();
            } else if *self.next_index.get(&from).unwrap_or(&1) <= self.last_index() {
                self.send_append_to(from);
            }
        } else {
            // Log mismatch: back off and retry (snapshot kicks in once next falls
            // to or below the compaction point). Ignore a delayed failure for
            // a prefix a newer response already proved replicated.
            if resp.match_index < *self.match_index.get(&from).unwrap_or(&0) {
                return;
            }
            let n = self.next_index.entry(from).or_insert(1);
            *n = (*n)
                .saturating_sub(1)
                .min(resp.match_index.saturating_add(1))
                .max(1);
            self.send_append_to(from);
        }
    }

    fn handle_install_snapshot(&mut self, req: InstallSnapshotReq) {
        if req.term < self.current_term {
            let (term, si) = (self.current_term, self.snapshot_index);
            self.send(
                req.leader,
                RaftMsg::InstallSnapshotResp(InstallSnapshotResp {
                    term,
                    accepted: false,
                    snapshot_index: si,
                }),
            );
            return;
        }
        self.step_down(req.term);
        self.leader_id = Some(req.leader);
        let accepted = if req.snapshot_index < self.snapshot_index {
            // A later state-machine snapshot supersedes this request.
            true
        } else if req.snapshot_index == self.snapshot_index {
            // Raft indexes are immutable identities. A retry is idempotent
            // only when its term and bytes are exactly the same.
            req.snapshot_term == self.snapshot_term && req.data == self.snapshot
        } else {
            // Keep a matching suffix when this follower was already at or
            // beyond the prospective snapshot point. This lets a leader first
            // prove that every voter accepted a checkpoint and only then drop
            // its own prefix, without deleting later entries on a caught-up
            // follower.
            let retained_suffix = if req.snapshot_index <= self.last_index()
                && self.term_at(req.snapshot_index) == req.snapshot_term
            {
                let first = (req.snapshot_index - self.snapshot_index) as usize;
                self.log[first.min(self.log.len())..].to_vec()
            } else {
                Vec::new()
            };
            self.log = retained_suffix;
            self.resident_log_bytes = self.log.iter().map(|entry| entry.command.len()).sum();
            self.snapshot_index = req.snapshot_index;
            self.snapshot_term = req.snapshot_term;
            self.snapshot = req.data.clone();
            self.installed_snapshot = Some(req.data);
            if self.commit_index < req.snapshot_index {
                self.commit_index = req.snapshot_index;
            }
            self.commit_index = self.commit_index.min(self.last_index());
            self.last_applied = req.snapshot_index;
            true
        };
        let (term, si) = (self.current_term, self.snapshot_index);
        self.send(
            req.leader,
            RaftMsg::InstallSnapshotResp(InstallSnapshotResp {
                term,
                accepted,
                snapshot_index: si,
            }),
        );
    }

    fn handle_install_snapshot_resp(&mut self, from: NodeId, resp: InstallSnapshotResp) {
        if resp.term > self.current_term {
            self.step_down(resp.term);
            return;
        }
        if self.role != Role::Leader || resp.term != self.current_term {
            return;
        }
        if !resp.accepted {
            return;
        }
        let m = resp.snapshot_index;
        if m > *self.match_index.get(&from).unwrap_or(&0) {
            self.match_index.insert(from, m);
        }
        self.next_index.insert(from, m + 1);
        let old = self.commit_index;
        self.maybe_commit();
        if self.commit_index > old {
            self.broadcast_append();
        } else if *self.next_index.get(&from).unwrap_or(&1) <= self.last_index() {
            self.send_append_to(from);
        }
    }

    /// Leader: advance `commit_index` to the highest index replicated to a
    /// majority of **voters** (both incoming and outgoing sets if joint)
    /// whose entry is from the current term.
    fn maybe_commit(&mut self) {
        if self.role != Role::Leader {
            return;
        }
        let last = self.last_index();
        let mut new_commit = self.commit_index;
        let incoming_maj = self.conf_state.membership.voters.len() / 2 + 1;
        let outgoing_maj = self
            .conf_state
            .outgoing
            .as_ref()
            .map(|out| out.len() / 2 + 1);

        for n in (self.commit_index + 1)..=last {
            if self.term_at(n) != self.current_term {
                continue;
            }
            let mut incoming_count = 0usize;
            for v in &self.conf_state.membership.voters {
                let m = if *v == self.id {
                    last
                } else {
                    *self.match_index.get(v).unwrap_or(&0)
                };
                if m >= n {
                    incoming_count += 1;
                }
            }
            let incoming_ok = incoming_count >= incoming_maj;

            let outgoing_ok = match &self.conf_state.outgoing {
                Some(outgoing) => {
                    let mut outgoing_count = 0usize;
                    for v in outgoing {
                        let m = if *v == self.id {
                            last
                        } else {
                            *self.match_index.get(v).unwrap_or(&0)
                        };
                        if m >= n {
                            outgoing_count += 1;
                        }
                    }
                    outgoing_count >= outgoing_maj.unwrap()
                }
                None => true,
            };

            if incoming_ok && outgoing_ok {
                new_commit = n;
            }
        }
        self.commit_index = new_commit;
    }

    fn handle_timeout_now(&mut self, req: TimeoutNowReq) {
        if !self.is_voter {
            return;
        }
        if req.term < self.current_term {
            return;
        }
        if req.term > self.current_term {
            self.current_term = req.term;
            self.voted_for = None;
        }
        self.start_election();
    }
}
// CODEGEN-END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_derives_election_timeout_from_public_floor_and_node_id() {
        let membership = auto_membership(3);
        let id = 7;

        let node = RaftNode::new(id, &membership);

        assert_eq!(node.election_timeout, ELECTION_TIMEOUT_FLOOR_TICKS + id);
    }

    #[test]
    fn leader_sends_first_periodic_heartbeat_at_public_interval() {
        let membership = Membership {
            voters: vec![1, 2],
            learners: Vec::new(),
        };
        let mut node = RaftNode::new(1, &membership);
        node.become_leader();
        assert_eq!(
            node.take_outgoing().len(),
            1,
            "leader sends its initial append"
        );

        for _ in 0..HEARTBEAT_INTERVAL_TICKS - 1 {
            node.tick();
            assert!(
                node.take_outgoing().is_empty(),
                "heartbeat must not be early"
            );
        }

        node.tick();
        assert_eq!(
            node.take_outgoing().len(),
            1,
            "heartbeat is due at the interval"
        );
    }

    #[test]
    fn public_timing_constants_are_nonzero_and_ordered() {
        assert_ne!(ELECTION_TIMEOUT_FLOOR_TICKS, 0);
        assert_ne!(HEARTBEAT_INTERVAL_TICKS, 0);
        assert!(HEARTBEAT_INTERVAL_TICKS < ELECTION_TIMEOUT_FLOOR_TICKS);
    }
}
