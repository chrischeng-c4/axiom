//! Self-contained single-shard Raft consensus core (no external dependency).
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
//! via [`take_committed`](RaftNode::take_committed) for the relay layer to apply
//! to its own engine — relay's append-only durable log is the state machine and
//! is never rewritten here.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Stable node identity (in k8s, the StatefulSet ordinal).
pub type NodeId = u64;
pub type Term = u64;
/// 1-based Raft log index; 0 means "before the first entry".
pub type Index = u64;

/// Logical ticks before a voter starts an election (distinct per node so the
/// deterministic simulation does not livelock on split votes).
const ELECTION_MIN: u64 = 10;
/// Ticks between leader heartbeats / replication pushes.
const HEARTBEAT_TIMEOUT: u64 = 3;

/// One replicated command entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaftEntry {
    pub term: Term,
    pub index: Index,
    pub command: Vec<u8>,
}

/// The durable hard state of a Raft node: what must survive a restart so the
/// node never double-votes in a term or forgets acknowledged entries.
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedState {
    pub term: Term,
    pub voted_for: Option<NodeId>,
    pub log: Vec<RaftEntry>,
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

#[derive(Clone, Debug)]
pub enum RaftMsg {
    Vote(VoteReq),
    VoteResp(VoteResp),
    Append(AppendReq),
    AppendResp(AppendResp),
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

/// Cluster membership for one Raft group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Membership {
    pub voters: Vec<NodeId>,
    pub learners: Vec<NodeId>,
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

/// A single-shard Raft participant.
///
pub struct RaftNode {
    id: NodeId,
    voters: Vec<NodeId>,
    peers: Vec<NodeId>, // all other members (voters + learners)
    is_voter: bool,

    role: Role,
    current_term: Term,
    voted_for: Option<NodeId>,
    log: Vec<RaftEntry>,
    commit_index: Index,
    last_applied: Index,

    // leader-only, per peer
    next_index: HashMap<NodeId, Index>,
    match_index: HashMap<NodeId, Index>,

    // election
    votes: HashSet<NodeId>,
    election_elapsed: u64,
    election_timeout: u64,
    heartbeat_elapsed: u64,
    /// Last known leader for this term (drives producer redirect-to-leader).
    leader_id: Option<NodeId>,

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
        RaftNode {
            id,
            voters: membership.voters.clone(),
            peers,
            is_voter: membership.voters.contains(&id),
            role: Role::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            votes: HashSet::new(),
            election_elapsed: 0,
            // distinct per node so one voter always times out first.
            election_timeout: ELECTION_MIN + id,
            heartbeat_elapsed: 0,
            leader_id: None,
            outbox: Vec::new(),
        }
    }

    /// Restore a node from durable [`PersistedState`]: term, votedFor and log
    /// are recovered; volatile state (role, commit/apply indices) restarts as a
    /// Follower and is re-derived via replication. Committed entries re-apply
    /// idempotently downstream.
    ///
    pub fn from_persisted(id: NodeId, membership: &Membership, state: PersistedState) -> RaftNode {
        let mut node = RaftNode::new(id, membership);
        node.current_term = state.term;
        node.voted_for = state.voted_for;
        node.log = state.log;
        node
    }

    /// Snapshot the durable hard state for [`crate::raft_store::RaftStore::save`].
    ///
    pub fn persisted(&self) -> PersistedState {
        PersistedState {
            term: self.current_term,
            voted_for: self.voted_for,
            log: self.log.clone(),
        }
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
    pub fn last_index(&self) -> Index {
        self.log.len() as Index
    }
    /// Last known leader for the current term (for producer redirect).
    pub fn leader(&self) -> Option<NodeId> {
        self.leader_id
    }

    fn last_term(&self) -> Term {
        self.log.last().map(|e| e.term).unwrap_or(0)
    }

    fn term_at(&self, index: Index) -> Term {
        if index == 0 {
            0
        } else {
            self.log
                .get((index - 1) as usize)
                .map(|e| e.term)
                .unwrap_or(0)
        }
    }

    fn majority(&self) -> usize {
        self.voters.len() / 2 + 1
    }

    /// Drain messages the driver must deliver.
    pub fn take_outgoing(&mut self) -> Vec<Outgoing> {
        std::mem::take(&mut self.outbox)
    }

    /// Newly committed entries (in index order); advances `last_applied`.
    pub fn take_committed(&mut self) -> Vec<RaftEntry> {
        let mut out = Vec::new();
        while self.last_applied < self.commit_index {
            let e = self.log[self.last_applied as usize].clone();
            out.push(e);
            self.last_applied += 1;
        }
        out
    }

    fn send(&mut self, to: NodeId, msg: RaftMsg) {
        self.outbox.push(Outgoing { to, msg });
    }

    /// Advance one logical tick: leaders heartbeat, voters may start an election.
    pub fn tick(&mut self) {
        self.election_elapsed += 1;
        self.heartbeat_elapsed += 1;
        if self.role == Role::Leader {
            if self.heartbeat_elapsed >= HEARTBEAT_TIMEOUT {
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
        let peers: Vec<NodeId> = self
            .voters
            .iter()
            .copied()
            .filter(|v| *v != self.id)
            .collect();
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
        let granted = self
            .votes
            .iter()
            .filter(|v| self.voters.contains(v))
            .count();
        if granted >= self.majority() {
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
        self.broadcast_append();
    }

    fn step_down(&mut self, term: Term) {
        if term > self.current_term {
            self.current_term = term;
            self.voted_for = None;
        }
        self.role = Role::Follower;
        self.election_elapsed = 0;
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
    ///
    pub fn propose(&mut self, command: Vec<u8>) -> Option<Index> {
        if self.role != Role::Leader {
            return None;
        }
        let index = self.last_index() + 1;
        self.log.push(RaftEntry {
            term: self.current_term,
            index,
            command,
        });
        self.broadcast_append();
        self.maybe_commit(); // sole voter commits immediately
        Some(index)
    }

    /// Feed an incoming message from `from`.
    ///
    pub fn handle(&mut self, from: NodeId, msg: RaftMsg) {
        match msg {
            RaftMsg::Vote(req) => self.handle_vote(from, req),
            RaftMsg::VoteResp(resp) => self.handle_vote_resp(from, resp),
            RaftMsg::Append(req) => self.handle_append(req),
            RaftMsg::AppendResp(resp) => self.handle_append_resp(from, resp),
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

        // Log matching: the entry preceding the new ones must agree.
        if req.prev_log_index > self.last_index()
            || self.term_at(req.prev_log_index) != req.prev_log_term
        {
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

        // Append, truncating any conflicting suffix.
        for e in &req.entries {
            let pos = (e.index - 1) as usize;
            if pos < self.log.len() {
                if self.log[pos].term != e.term {
                    self.log.truncate(pos);
                    self.log.push(e.clone());
                }
            } else {
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
            self.match_index.insert(from, resp.match_index);
            self.next_index.insert(from, resp.match_index + 1);
            let old = self.commit_index;
            self.maybe_commit();
            if self.commit_index > old {
                // Propagate the new commit to everyone.
                self.broadcast_append();
            } else if *self.next_index.get(&from).unwrap_or(&1) <= self.last_index() {
                self.send_append_to(from);
            }
        } else {
            // Log mismatch: back off and retry.
            let n = self.next_index.entry(from).or_insert(1);
            *n = (*n).saturating_sub(1).max(1);
            self.send_append_to(from);
        }
    }

    /// Leader: advance `commit_index` to the highest index replicated to a
    /// majority of **voters** whose entry is from the current term.
    fn maybe_commit(&mut self) {
        if self.role != Role::Leader {
            return;
        }
        let last = self.last_index();
        let mut new_commit = self.commit_index;
        for n in (self.commit_index + 1)..=last {
            if self.term_at(n) != self.current_term {
                continue;
            }
            let mut count = 0usize;
            for v in &self.voters {
                let m = if *v == self.id {
                    last
                } else {
                    *self.match_index.get(v).unwrap_or(&0)
                };
                if m >= n {
                    count += 1;
                }
            }
            if count >= self.majority() {
                new_commit = n;
            }
        }
        self.commit_index = new_commit;
    }
}
