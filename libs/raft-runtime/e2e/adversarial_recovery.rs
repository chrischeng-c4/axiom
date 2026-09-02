//! Replayable adversarial recovery tests for the deterministic Raft host.
//!
//! A trace is the sole host-operation log. Messages only move by a recorded
//! `Take` into an opaque mailbox and a later recorded `Deliver`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use raft_core::{Index, Membership, NodeId, RaftNode};
use raft_runtime::conformance::{
    ConformanceRole, DeterministicHost, EnvelopeKind, PendingEnvelope, StateMachineOperation,
    TRACE_SCHEMA,
};
use raft_runtime::{FsyncPolicy, RaftStateMachine, RaftStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const SEEDS: [u64; 2] = [1, 0x9e37_79b9_7f4a_7c15];
const MAX_ACTIONS: usize = 192;
const MAX_PENDING: usize = 128;
const MAX_RESTARTS: usize = 2;
const MAX_COMPACTIONS: usize = 2;
const MAX_DDMIN_REPLAYS: usize = 2048;
const REPLAY_MAX_BYTES: usize = 1024 * 1024;

/// Snapshots retain the full applied `(Index, command)` prefix. Config entries
/// can create index gaps, so a plain applied watermark is not enough evidence.
#[derive(Default)]
struct Sm {
    prefix: Mutex<Vec<(Index, Vec<u8>)>>,
}

impl Sm {
    fn prefix(&self) -> Vec<(Index, Vec<u8>)> {
        self.prefix.lock().expect("state-machine lock").clone()
    }
}

impl RaftStateMachine for Sm {
    fn apply(&self, index: Index, command: &[u8]) -> anyhow::Result<()> {
        let mut prefix = self.prefix.lock().expect("state-machine lock");
        match prefix.last() {
            Some((last, saved)) if *last == index => {
                anyhow::ensure!(saved == command, "same index has different command");
            }
            Some((last, _)) => {
                anyhow::ensure!(
                    index > *last,
                    "state machine applied index {index} after later index {last}"
                );
                prefix.push((index, command.to_vec()));
            }
            None => prefix.push((index, command.to_vec())),
        }
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        serde_json::to_writer(writer, &self.prefix()).map_err(Into::into)
    }

    fn restore(&self, reader: &mut dyn Read) -> anyhow::Result<()> {
        let prefix: Vec<(Index, Vec<u8>)> = serde_json::from_reader(reader)?;
        for pair in prefix.windows(2) {
            anyhow::ensure!(
                pair[0].0 < pair[1].0,
                "snapshot state-machine prefix is not strictly ordered"
            );
        }
        *self.prefix.lock().expect("state-machine lock") = prefix;
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.prefix().last().map(|(index, _)| *index).unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Template {
    ElectionSplit,
    ReplicationPartition,
    StaleAppendResp,
    DurableRestart,
    LaggingSnapshot,
    ThreeToFourMembership,
    FourToFiveMembership,
    DemoteRemoveRecovery,
}

impl Template {
    const ALL: [Self; 8] = [
        Self::ElectionSplit,
        Self::ReplicationPartition,
        Self::StaleAppendResp,
        Self::DurableRestart,
        Self::LaggingSnapshot,
        Self::ThreeToFourMembership,
        Self::FourToFiveMembership,
        Self::DemoteRemoveRecovery,
    ];

    fn topology(self) -> (usize, usize) {
        match self {
            Self::ThreeToFourMembership => (3, 4),
            Self::FourToFiveMembership => (4, 5),
            Self::DemoteRemoveRecovery => (4, 4),
            _ => (3, 3),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TraceInit {
    voters: usize,
    nodes: usize,
    template: Template,
    seed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Replay {
    schema: String,
    init: TraceInit,
    actions: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WireKind {
    Vote,
    VoteResponse,
    Append,
    AppendResponse,
    InstallSnapshot,
    InstallSnapshotResponse,
    TimeoutNow,
}

impl From<EnvelopeKind> for WireKind {
    fn from(kind: EnvelopeKind) -> Self {
        match kind {
            EnvelopeKind::Vote => Self::Vote,
            EnvelopeKind::VoteResponse => Self::VoteResponse,
            EnvelopeKind::Append => Self::Append,
            EnvelopeKind::AppendResponse => Self::AppendResponse,
            EnvelopeKind::InstallSnapshot => Self::InstallSnapshot,
            EnvelopeKind::InstallSnapshotResponse => Self::InstallSnapshotResponse,
            EnvelopeKind::TimeoutNow => Self::TimeoutNow,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    Tick {
        node: NodeId,
    },
    Take {
        from: NodeId,
        to: NodeId,
        id: u64,
        kind: WireKind,
        fingerprint: String,
    },
    Deliver {
        id: u64,
    },
    Propose {
        node: NodeId,
        command: Vec<u8>,
    },
    AddLearner {
        node: NodeId,
        peer: NodeId,
    },
    Promote {
        node: NodeId,
        peer: NodeId,
    },
    Demote {
        node: NodeId,
        peer: NodeId,
    },
    Remove {
        node: NodeId,
        peer: NodeId,
    },
    Restart {
        node: NodeId,
    },
    Compact {
        node: NodeId,
    },
    Checkpoint {
        checkpoint: Checkpoint,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "checkpoint", rename_all = "snake_case", deny_unknown_fields)]
enum Checkpoint {
    /// Both candidates are live, but no term has a leader yet.
    ElectionSplit { first: NodeId, second: NodeId },
    /// `newer` is delivered before the held real `old` response.
    StaleAppendResponse { old: u64, newer: u64 },
    /// The leave-joint acknowledgement witness reaches exactly one side.
    JointOneSide {
        leader: NodeId,
        outgoing: Vec<NodeId>,
        incoming: Vec<NodeId>,
        acknowledged: Vec<NodeId>,
        target_index: Index,
        append_ids: Vec<u64>,
        response_ids: Vec<u64>,
    },
    /// A real opaque InstallSnapshot envelope reached this follower.
    SnapshotDelivered { node: NodeId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SafetyViolation {
    ElectionSafety {
        term: u64,
        first: NodeId,
        second: NodeId,
    },
    StateMachinePrefix {
        first: NodeId,
        second: NodeId,
    },
    CommandConflict {
        index: Index,
    },
    StateMachineAhead {
        node: NodeId,
    },
    RestartPrefix {
        node: NodeId,
    },
    RestartMembership {
        node: NodeId,
    },
    StaleAppendResponse {
        old: u64,
        newer: u64,
    },
    JointQuorum {
        template: Template,
    },
    SnapshotNotDelivered {
        node: NodeId,
    },
    SnapshotPrefix {
        node: NodeId,
    },
    FinalMembership {
        template: Template,
        node: NodeId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ReplayFailure {
    Input(String),
    Step { step: usize, message: String },
    Safety(SafetyViolation),
}

impl fmt::Display for ReplayFailure {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message) => write!(out, "input: {message}"),
            Self::Step { step, message } => write!(out, "step {step}: {message}"),
            Self::Safety(violation) => write!(out, "safety: {violation:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutant {
    OneSidedJointQuorum,
    RegressOnlyStaleAppendResponse,
    IgnorePersistedMembership,
}

#[derive(Clone)]
struct RestartExpectation {
    node: NodeId,
    prefix: Vec<(Index, Vec<u8>)>,
    membership: Membership,
    reopen_adapter_membership: Membership,
    conf_state_before_reopen: bool,
    conf_state_after_reopen: bool,
}

struct StaleRank {
    old: u64,
    newer: u64,
    newer_delivered: bool,
    old_after_new: bool,
    rank_before_old: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeliveredMeta {
    from: NodeId,
    to: NodeId,
    kind: EnvelopeKind,
}

struct Engine {
    init: TraceInit,
    dir: TempDir,
    hosts: Vec<DeterministicHost>,
    sms: Vec<Arc<Sm>>,
    mailbox: BTreeMap<u64, PendingEnvelope>,
    recording: bool,
    actions: Vec<Action>,
    restarts: usize,
    compactions: usize,
    fault: Option<Mutant>,
    leader_by_term: BTreeMap<u64, NodeId>,
    command_by_index: BTreeMap<Index, Vec<u8>>,
    last_commit: Vec<Index>,
    restart_expectation: Option<RestartExpectation>,
    delivered: BTreeSet<u64>,
    delivered_append_responses: BTreeMap<u64, (NodeId, NodeId)>,
    delivered_meta: BTreeMap<u64, DeliveredMeta>,
    delivered_at: BTreeMap<u64, usize>,
    append_recipient_last_index: BTreeMap<u64, Index>,
    delivered_snapshots: BTreeSet<NodeId>,
    snapshot_lagger: Option<NodeId>,
    stale: Option<StaleRank>,
    replication_rank: u64,
    executed_actions: usize,
}

impl Engine {
    fn new(init: TraceInit, recording: bool, fault: Option<Mutant>) -> Result<Self, ReplayFailure> {
        let dir = TempDir::new().map_err(|error| ReplayFailure::Step {
            step: 0,
            message: format!("temporary store: {error}"),
        })?;
        let membership = Membership {
            voters: (0..init.voters as NodeId).collect(),
            learners: Vec::new(),
        };
        let mut hosts = Vec::with_capacity(init.nodes);
        let mut sms = Vec::with_capacity(init.nodes);
        for id in 0..init.nodes as NodeId {
            let sm = Arc::new(Sm::default());
            let store = RaftStore::open(
                dir.path().to_str().expect("temporary path is UTF-8"),
                id,
                FsyncPolicy::Always,
            )
            .map_err(|error| ReplayFailure::Step {
                step: 0,
                message: format!("open node {id}: {error}"),
            })?;
            hosts.push(
                DeterministicHost::open_with_envelope_epoch(
                    id,
                    membership.clone(),
                    store,
                    sm.clone(),
                    id as u32,
                )
                .map_err(|error| ReplayFailure::Step {
                    step: 0,
                    message: format!("start node {id}: {error}"),
                })?,
            );
            sms.push(sm);
        }
        Ok(Self {
            last_commit: vec![0; init.nodes],
            init,
            dir,
            hosts,
            sms,
            mailbox: BTreeMap::new(),
            recording,
            actions: Vec::new(),
            restarts: 0,
            compactions: 0,
            fault,
            leader_by_term: BTreeMap::new(),
            command_by_index: BTreeMap::new(),
            restart_expectation: None,
            delivered: BTreeSet::new(),
            delivered_append_responses: BTreeMap::new(),
            delivered_meta: BTreeMap::new(),
            delivered_at: BTreeMap::new(),
            append_recipient_last_index: BTreeMap::new(),
            delivered_snapshots: BTreeSet::new(),
            snapshot_lagger: None,
            stale: None,
            replication_rank: 0,
            executed_actions: 0,
        })
    }

    fn replay(&self) -> Replay {
        Replay {
            schema: TRACE_SCHEMA.to_owned(),
            init: self.init.clone(),
            actions: self.actions.clone(),
        }
    }

    fn node_index(&self, node: NodeId) -> Result<usize, String> {
        let index = usize::try_from(node).map_err(|_| "unknown node".to_owned())?;
        (index < self.hosts.len())
            .then_some(index)
            .ok_or_else(|| format!("unknown node {node}"))
    }

    /// Reserve the action before any host or mailbox mutation.  This is what
    /// prevents a 193rd mutation from escaping the serialized trace.
    fn begin_action(&mut self) -> Result<usize, ReplayFailure> {
        if self.recording && self.actions.len() >= MAX_ACTIONS {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("trace exceeds {MAX_ACTIONS} actions"),
            });
        }
        let step = self.executed_actions;
        self.executed_actions += 1;
        Ok(step)
    }

    fn note(&mut self, action: Action) {
        if self.recording {
            self.actions.push(action);
        }
    }

    fn step_error(&self, error: impl fmt::Display) -> ReplayFailure {
        ReplayFailure::Step {
            step: self.actions.len(),
            message: error.to_string(),
        }
    }

    fn tick(&mut self, node: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .tick()
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::Tick { node });
        self.observe()
    }

    fn take(&mut self, from: NodeId, to: NodeId) -> Result<u64, ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(from)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.node_index(to).map_err(|message| ReplayFailure::Step {
            step: self.actions.len(),
            message,
        })?;
        let envelope = self.hosts[index]
            .take_next(to)
            .ok_or_else(|| ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("no pending envelope from {from} to {to}"),
            })?;
        let meta = envelope.meta().clone();
        let id = meta.id;
        if meta.from != from || meta.to != to || self.mailbox.insert(id, envelope).is_some() {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: "invalid live envelope".to_owned(),
            });
        }
        self.note(Action::Take {
            from,
            to,
            id,
            kind: meta.kind.into(),
            fingerprint: meta.fingerprint,
        });
        self.pending_bound()?;
        Ok(id)
    }

    fn replay_take(
        &mut self,
        from: NodeId,
        to: NodeId,
        id: u64,
        kind: WireKind,
        fingerprint: String,
    ) -> Result<(), ReplayFailure> {
        let actual = self.take(from, to)?;
        let meta = self.mailbox.get(&actual).expect("taken envelope").meta();
        if actual != id || WireKind::from(meta.kind) != kind || meta.fingerprint != fingerprint {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("take metadata differs for envelope {id}"),
            });
        }
        Ok(())
    }

    fn deliver(&mut self, id: u64) -> Result<(), ReplayFailure> {
        let delivery_step = self.begin_action()?;
        let envelope = self
            .mailbox
            .remove(&id)
            .ok_or_else(|| ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("dangling envelope {id}"),
            })?;
        let meta = envelope.meta().clone();
        let target = self
            .node_index(meta.to)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[target]
            .receive(envelope)
            .map_err(|error| self.step_error(error))?;
        if meta.kind == EnvelopeKind::Append {
            self.append_recipient_last_index
                .insert(id, self.hosts[target].view().last_index);
        }
        if meta.kind == EnvelopeKind::InstallSnapshot {
            self.delivered_snapshots.insert(meta.to);
        }
        self.delivered_meta.insert(
            id,
            DeliveredMeta {
                from: meta.from,
                to: meta.to,
                kind: meta.kind,
            },
        );
        self.delivered_at.insert(id, delivery_step);
        if meta.kind == EnvelopeKind::AppendResponse {
            self.delivered_append_responses
                .insert(id, (meta.from, meta.to));
            if let Some(stale) = self.stale.as_mut() {
                if id == stale.newer {
                    stale.newer_delivered = true;
                }
                if id == stale.old && stale.newer_delivered {
                    stale.old_after_new = true;
                    if self.fault == Some(Mutant::RegressOnlyStaleAppendResponse) {
                        self.replication_rank = self.replication_rank.saturating_sub(1);
                    } else {
                        self.replication_rank += 1;
                    }
                } else {
                    self.replication_rank += 1;
                }
            } else {
                self.replication_rank += 1;
            }
        }
        self.delivered.insert(id);
        self.note(Action::Deliver { id });
        self.pending_bound()?;
        self.observe()?;
        if let Some(stale) = &self.stale {
            if stale.old_after_new && self.replication_rank < stale.rank_before_old {
                return Err(ReplayFailure::Safety(
                    SafetyViolation::StaleAppendResponse {
                        old: stale.old,
                        newer: stale.newer,
                    },
                ));
            }
        }
        Ok(())
    }

    fn propose(&mut self, node: NodeId, command: Vec<u8>) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .try_propose(StateMachineOperation::Command(command.clone()))
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::Propose { node, command });
        self.observe()
    }

    fn add_learner(&mut self, node: NodeId, peer: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.node_index(peer)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .try_add_learner(peer)
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::AddLearner { node, peer });
        self.observe()
    }

    fn promote(&mut self, node: NodeId, peer: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .promote_learner(peer)
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::Promote { node, peer });
        self.observe()
    }

    fn demote(&mut self, node: NodeId, peer: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .demote_voter(peer)
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::Demote { node, peer });
        self.observe()
    }

    fn remove(&mut self, node: NodeId, peer: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .remove_member(peer)
            .map_err(|error| self.step_error(error))?;
        self.pending_bound()?;
        self.note(Action::Remove { node, peer });
        self.observe()
    }

    fn restart(&mut self, node: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        if self.restarts >= MAX_RESTARTS {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("trace exceeds {MAX_RESTARTS} restarts"),
            });
        }
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        let membership = self.hosts[index].view().membership;
        let prefix = self.sms[index].prefix();
        let store = RaftStore::open(
            self.dir.path().to_str().expect("temporary path is UTF-8"),
            node,
            FsyncPolicy::Always,
        )
        .map_err(|error| self.step_error(error))?;
        let persisted = store
            .load()
            .map_err(|error| self.step_error(error))?
            .ok_or_else(|| ReplayFailure::Step {
                step: self.actions.len(),
                message: "restart has no durable state".to_owned(),
            })?;
        let conf_state_before_reopen = persisted.conf.is_some();
        if !conf_state_before_reopen {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: "restart lost persisted ConfState before reopen".to_owned(),
            });
        }
        // The normal host reopens the intact durable image. The mutant has a
        // separate test-only adapter that clones the state, drops only its
        // clone's ConfState, and invokes the public core recovery path with a
        // conflicting bootstrap. It therefore observes a real wrong recovered
        // membership without writing or changing the actual RaftStore.
        let reopen_adapter_membership = if self.fault == Some(Mutant::IgnorePersistedMembership) {
            let conflicting_bootstrap = Membership {
                voters: vec![node],
                learners: Vec::new(),
            };
            let mut state_without_conf = persisted.clone();
            state_without_conf.conf = None;
            RaftNode::from_persisted(node, &conflicting_bootstrap, state_without_conf)
                .conf_state()
                .membership
                .clone()
        } else {
            membership.clone()
        };
        let sm = Arc::new(Sm::default());
        let epoch = ((self.restarts as u32 + 1) << 16) | node as u32;
        self.hosts[index] = DeterministicHost::open_with_envelope_epoch(
            node,
            membership.clone(),
            store,
            sm.clone(),
            epoch,
        )
        .map_err(|error| self.step_error(error))?;
        let conf_state_after_reopen = self.hosts[index]
            .store()
            .load()
            .map_err(|error| self.step_error(error))?
            .is_some_and(|state| state.conf.is_some());
        if !conf_state_after_reopen {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: "restart reopen cleared persisted ConfState".to_owned(),
            });
        }
        self.sms[index] = sm;
        self.restarts += 1;
        self.restart_expectation = Some(RestartExpectation {
            node,
            prefix,
            membership,
            reopen_adapter_membership,
            conf_state_before_reopen,
            conf_state_after_reopen,
        });
        self.pending_bound()?;
        self.note(Action::Restart { node });
        self.observe()
    }

    fn compact(&mut self, node: NodeId) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        if self.compactions >= MAX_COMPACTIONS {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("trace exceeds {MAX_COMPACTIONS} compactions"),
            });
        }
        let index = self
            .node_index(node)
            .map_err(|message| ReplayFailure::Step {
                step: self.actions.len(),
                message,
            })?;
        self.hosts[index]
            .snapshot_and_compact()
            .map_err(|error| self.step_error(error))?;
        self.compactions += 1;
        self.pending_bound()?;
        self.note(Action::Compact { node });
        self.observe()
    }

    fn checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), ReplayFailure> {
        self.begin_action()?;
        match &checkpoint {
            Checkpoint::ElectionSplit { first, second } => {
                let first = self
                    .node_index(*first)
                    .map_err(|message| ReplayFailure::Step {
                        step: self.actions.len(),
                        message,
                    })?;
                let second = self
                    .node_index(*second)
                    .map_err(|message| ReplayFailure::Step {
                        step: self.actions.len(),
                        message,
                    })?;
                if self
                    .hosts
                    .iter()
                    .any(|host| host.view().role == ConformanceRole::Leader)
                    || self.hosts[first].view().role != ConformanceRole::Candidate
                    || self.hosts[second].view().role != ConformanceRole::Candidate
                {
                    return Err(ReplayFailure::Step {
                        step: self.actions.len(),
                        message: "election split window is not two candidates without a leader"
                            .to_owned(),
                    });
                }
            }
            Checkpoint::StaleAppendResponse { old, newer } => {
                if !self.delivered_append_responses.contains_key(newer)
                    || self.delivered.contains(old)
                    || self.mailbox.get(old).map(|envelope| envelope.meta().kind)
                        != Some(EnvelopeKind::AppendResponse)
                {
                    return Err(ReplayFailure::Step {
                        step: self.actions.len(),
                        message: "stale checkpoint has no old-after-new witness".to_owned(),
                    });
                }
                self.stale = Some(StaleRank {
                    old: *old,
                    newer: *newer,
                    newer_delivered: true,
                    old_after_new: false,
                    rank_before_old: self.replication_rank,
                });
            }
            Checkpoint::JointOneSide {
                leader,
                outgoing,
                incoming,
                acknowledged,
                target_index,
                append_ids,
                response_ids,
            } => {
                let leader_index =
                    self.node_index(*leader)
                        .map_err(|message| ReplayFailure::Step {
                            step: self.actions.len(),
                            message,
                        })?;
                let witness: BTreeSet<_> = acknowledged.iter().copied().collect();
                if witness.len() != acknowledged.len()
                    || append_ids.len() != response_ids.len()
                    || response_ids.len() + 1 != acknowledged.len()
                    || !witness.contains(leader)
                    || !append_ids
                        .iter()
                        .zip(response_ids)
                        .zip(acknowledged.iter().skip(1))
                        .all(|((append_id, response_id), node)| {
                            let Some(append) = self.delivered_meta.get(append_id) else {
                                return false;
                            };
                            let Some(response) = self.delivered_meta.get(response_id) else {
                                return false;
                            };
                            let Some(append_step) = self.delivered_at.get(append_id) else {
                                return false;
                            };
                            let Some(response_step) = self.delivered_at.get(response_id) else {
                                return false;
                            };
                            let Some(recipient_last_index) =
                                self.append_recipient_last_index.get(append_id)
                            else {
                                return false;
                            };
                            append.from == *leader
                                && append.to == *node
                                && append.kind == EnvelopeKind::Append
                                && *recipient_last_index >= *target_index
                                && response.from == *node
                                && response.to == *leader
                                && response.kind == EnvelopeKind::AppendResponse
                                && append_step < response_step
                                && !self.delivered_meta.iter().any(|(other_id, other)| {
                                    other_id != append_id
                                        && other.from == *leader
                                        && other.to == *node
                                        && other.kind == EnvelopeKind::Append
                                        && self.delivered_at.get(other_id).is_some_and(|step| {
                                            step > append_step && step < response_step
                                        })
                                })
                        })
                {
                    return Err(ReplayFailure::Step {
                        step: self.actions.len(),
                        message: "joint checkpoint has no response witness".to_owned(),
                    });
                }
                let quorum = |voters: &[NodeId]| {
                    witness.iter().filter(|node| voters.contains(node)).count()
                        >= voters.len() / 2 + 1
                };
                let old_ok = quorum(outgoing);
                let new_ok = quorum(incoming);
                if old_ok == new_ok {
                    return Err(ReplayFailure::Step {
                        step: self.actions.len(),
                        message: "joint schedule must satisfy exactly one quorum".to_owned(),
                    });
                }
                let model_accepted = match self.fault {
                    Some(Mutant::OneSidedJointQuorum)
                        if self.init.template == Template::ThreeToFourMembership =>
                    {
                        old_ok
                    }
                    Some(Mutant::OneSidedJointQuorum)
                        if self.init.template == Template::FourToFiveMembership =>
                    {
                        new_ok
                    }
                    _ => old_ok && new_ok,
                };
                let host_accepted = self.hosts[leader_index].view().commit_index >= *target_index;
                if model_accepted != host_accepted {
                    return Err(ReplayFailure::Safety(SafetyViolation::JointQuorum {
                        template: self.init.template,
                    }));
                }
            }
            Checkpoint::SnapshotDelivered { node } => {
                if !self.delivered_snapshots.contains(node) {
                    return Err(ReplayFailure::Safety(
                        SafetyViolation::SnapshotNotDelivered { node: *node },
                    ));
                }
                self.snapshot_lagger = Some(*node);
            }
        }
        self.note(Action::Checkpoint { checkpoint });
        self.observe()
    }

    /// Drain is only repeated recorded `Take` then `Deliver` actions.
    fn drain(&mut self) -> Result<(), ReplayFailure> {
        loop {
            let mut progressed = false;
            for from in 0..self.hosts.len() as NodeId {
                let peers = self.hosts[self.node_index(from).expect("valid source")].ready_peers();
                for to in peers {
                    while self.hosts[self.node_index(from).expect("valid source")]
                        .ready_peers()
                        .contains(&to)
                    {
                        let id = self.take(from, to)?;
                        self.deliver(id)?;
                        progressed = true;
                    }
                }
            }
            if !progressed {
                return self.pending_bound();
            }
        }
    }

    /// This moves all host queues into the trace mailbox. It never delivers.
    fn take_all(&mut self) -> Result<(), ReplayFailure> {
        for from in 0..self.hosts.len() as NodeId {
            loop {
                let peers = self.hosts[self.node_index(from).expect("valid source")].ready_peers();
                if peers.is_empty() {
                    break;
                }
                for to in peers {
                    while self.hosts[self.node_index(from).expect("valid source")]
                        .ready_peers()
                        .contains(&to)
                    {
                        self.take(from, to)?;
                    }
                }
            }
        }
        self.pending_bound()
    }

    fn deliver_matching(
        &mut self,
        mut predicate: impl FnMut(&PendingEnvelope) -> bool,
    ) -> Result<Vec<u64>, ReplayFailure> {
        let ids: Vec<_> = self
            .mailbox
            .iter()
            .filter_map(|(&id, envelope)| predicate(envelope).then_some(id))
            .collect();
        for id in &ids {
            self.deliver(*id)?;
        }
        Ok(ids)
    }

    fn release_all(&mut self) -> Result<(), ReplayFailure> {
        loop {
            let held: Vec<_> = self.mailbox.keys().copied().collect();
            for id in held {
                self.deliver(id)?;
            }
            self.take_all()?;
            if self.mailbox.is_empty() && self.hosts.iter().all(|host| host.pending_len() == 0) {
                return Ok(());
            }
        }
    }

    fn leader(&self) -> Result<NodeId, ReplayFailure> {
        self.hosts
            .iter()
            .find(|host| host.view().role == ConformanceRole::Leader)
            .map(|host| host.view().id)
            .ok_or_else(|| ReplayFailure::Step {
                step: self.actions.len(),
                message: "no leader".to_owned(),
            })
    }

    fn elect(&mut self, candidate: NodeId) -> Result<NodeId, ReplayFailure> {
        for _ in 0..(50 + candidate) {
            self.tick(candidate)?;
        }
        self.drain()?;
        self.leader()
    }

    fn pending_ids(&self, mut predicate: impl FnMut(&PendingEnvelope) -> bool) -> Vec<u64> {
        self.mailbox
            .iter()
            .filter_map(|(&id, envelope)| predicate(envelope).then_some(id))
            .collect()
    }

    fn pending_bound(&self) -> Result<(), ReplayFailure> {
        let queued: usize = self.hosts.iter().map(DeterministicHost::pending_len).sum();
        if queued + self.mailbox.len() > MAX_PENDING {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: format!("trace exceeds {MAX_PENDING} pending envelopes"),
            });
        }
        Ok(())
    }

    fn observe(&mut self) -> Result<(), ReplayFailure> {
        for (offset, host) in self.hosts.iter().enumerate() {
            let view = host.view();
            if view.role == ConformanceRole::Leader {
                if let Some(first) = self.leader_by_term.insert(view.term, view.id) {
                    if first != view.id {
                        return Err(ReplayFailure::Safety(SafetyViolation::ElectionSafety {
                            term: view.term,
                            first,
                            second: view.id,
                        }));
                    }
                }
            }
            if view.commit_index < self.last_commit[offset] {
                return Err(ReplayFailure::Step {
                    step: self.actions.len(),
                    message: format!("commit regressed on node {}", view.id),
                });
            }
            self.last_commit[offset] = view.commit_index;
            if self.sms[offset].applied_index() > view.commit_index {
                return Err(ReplayFailure::Safety(SafetyViolation::StateMachineAhead {
                    node: view.id,
                }));
            }
            for (index, command) in self.sms[offset].prefix() {
                if let Some(saved) = self.command_by_index.get(&index) {
                    if saved != &command {
                        return Err(ReplayFailure::Safety(SafetyViolation::CommandConflict {
                            index,
                        }));
                    }
                } else {
                    self.command_by_index.insert(index, command);
                }
            }
        }
        for first in 0..self.sms.len() {
            for second in first + 1..self.sms.len() {
                let first_prefix = self.sms[first].prefix();
                let second_prefix = self.sms[second].prefix();
                let (shorter, longer) = if first_prefix.len() <= second_prefix.len() {
                    (&first_prefix, &second_prefix)
                } else {
                    (&second_prefix, &first_prefix)
                };
                if longer.get(..shorter.len()) != Some(shorter.as_slice()) {
                    return Err(ReplayFailure::Safety(SafetyViolation::StateMachinePrefix {
                        first: first as NodeId,
                        second: second as NodeId,
                    }));
                }
            }
        }
        if let Some(expected) = self.restart_expectation.take() {
            let node = self
                .node_index(expected.node)
                .expect("restart node remains valid");
            if self.sms[node].prefix() != expected.prefix {
                return Err(ReplayFailure::Safety(SafetyViolation::RestartPrefix {
                    node: expected.node,
                }));
            }
            if self.hosts[node].view().membership != expected.membership {
                return Err(ReplayFailure::Safety(SafetyViolation::RestartMembership {
                    node: expected.node,
                }));
            }
            if !expected.conf_state_before_reopen || !expected.conf_state_after_reopen {
                return Err(ReplayFailure::Step {
                    step: self.actions.len(),
                    message: "restart ConfState presence changed across reopen".to_owned(),
                });
            }
            if expected.reopen_adapter_membership != expected.membership {
                return Err(ReplayFailure::Safety(SafetyViolation::RestartMembership {
                    node: expected.node,
                }));
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), ReplayFailure> {
        if !self.mailbox.is_empty() || self.hosts.iter().any(|host| host.pending_len() != 0) {
            return Err(ReplayFailure::Step {
                step: self.actions.len(),
                message: "trace has dangling envelopes".to_owned(),
            });
        }
        if self.init.template == Template::StaleAppendResp {
            let stale = self.stale.as_ref().ok_or_else(|| ReplayFailure::Step {
                step: self.actions.len(),
                message: "stale checkpoint was not reached".to_owned(),
            })?;
            if !stale.old_after_new {
                return Err(ReplayFailure::Safety(
                    SafetyViolation::StaleAppendResponse {
                        old: stale.old,
                        newer: stale.newer,
                    },
                ));
            }
        }
        if let Some(lagger) = self.snapshot_lagger {
            let leader = self.leader()?;
            let lagger_index = self.node_index(lagger).expect("lagger node is valid");
            let leader_index = self.node_index(leader).expect("leader node is valid");
            if self.sms[lagger_index].prefix() != self.sms[leader_index].prefix() {
                return Err(ReplayFailure::Safety(SafetyViolation::SnapshotPrefix {
                    node: lagger,
                }));
            }
        }
        let expected = match self.init.template {
            Template::ThreeToFourMembership => Some((vec![0, 1, 2, 3], Vec::<NodeId>::new(), None)),
            Template::FourToFiveMembership => {
                Some((vec![0, 1, 2, 3, 4], Vec::<NodeId>::new(), None))
            }
            Template::DemoteRemoveRecovery => {
                let removed = if self.init.seed == SEEDS[0] { 1 } else { 0 };
                Some((
                    (0..4).filter(|node| *node != removed).collect(),
                    Vec::<NodeId>::new(),
                    Some(removed),
                ))
            }
            _ => None,
        };
        if let Some((voters, learners, removed)) = expected {
            for host in &self.hosts {
                let view = host.view();
                if let Some(removed) = removed {
                    if view.membership.voters.contains(&removed)
                        || view.membership.learners.contains(&removed)
                    {
                        return Err(ReplayFailure::Safety(SafetyViolation::FinalMembership {
                            template: self.init.template,
                            node: view.id,
                        }));
                    }
                }
                if Some(view.id) == removed {
                    continue;
                }
                if view.joint
                    || view.membership.voters != voters
                    || view.membership.learners != learners
                {
                    return Err(ReplayFailure::Safety(SafetyViolation::FinalMembership {
                        template: self.init.template,
                        node: view.id,
                    }));
                }
            }
        }
        self.observe()
    }
}

fn seed_candidate(seed: u64) -> NodeId {
    if seed == SEEDS[0] {
        0
    } else {
        1
    }
}

fn command(template: Template, seed: u64, serial: u8) -> Vec<u8> {
    let mut command = vec![template as u8, serial];
    command.extend_from_slice(&seed.to_le_bytes());
    command
}

fn build_trace(
    template: Template,
    seed: u64,
    fault: Option<Mutant>,
) -> Result<Replay, ReplayFailure> {
    let (voters, nodes) = template.topology();
    let mut engine = Engine::new(
        TraceInit {
            voters,
            nodes,
            template,
            seed,
        },
        true,
        fault,
    )?;
    let candidate = seed_candidate(seed);
    match template {
        Template::ElectionSplit => {
            let first = candidate;
            let second = (candidate + 1) % 3;
            for _ in 0..(50 + first) {
                engine.tick(first)?;
            }
            engine.take_all()?;
            for _ in 0..(50 + second) {
                engine.tick(second)?;
            }
            engine.take_all()?;
            engine.checkpoint(Checkpoint::ElectionSplit { first, second })?;
            engine.release_all()?;
            engine.leader()?;
        }
        Template::ReplicationPartition => {
            let leader = engine.elect(candidate)?;
            let isolated = if seed == SEEDS[0] {
                (leader + 1) % 3
            } else {
                (leader + 2) % 3
            };
            engine.propose(leader, command(template, seed, 0))?;
            engine.take_all()?;
            engine.deliver_matching(|envelope| envelope.meta().to != isolated)?;
            engine.take_all()?;
            engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::AppendResponse
                    && envelope.meta().from != isolated
            })?;
            engine.release_all()?;
        }
        Template::StaleAppendResp => {
            let leader = engine.elect(candidate)?;
            let follower = if seed == SEEDS[0] {
                (leader + 1) % 3
            } else {
                (leader + 2) % 3
            };
            engine.propose(leader, command(template, seed, 0))?;
            engine.take_all()?;
            engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::Append && envelope.meta().to == follower
            })?;
            engine.take_all()?;
            let old = *engine
                .pending_ids(|envelope| {
                    envelope.meta().kind == EnvelopeKind::AppendResponse
                        && envelope.meta().from == follower
                        && envelope.meta().to == leader
                })
                .first()
                .ok_or_else(|| ReplayFailure::Step {
                    step: engine.actions.len(),
                    message: "old AppendResp was not produced".to_owned(),
                })?;
            engine.propose(leader, command(template, seed, 1))?;
            engine.take_all()?;
            engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::Append && envelope.meta().to == follower
            })?;
            engine.take_all()?;
            let newer = *engine
                .pending_ids(|envelope| {
                    envelope.meta().kind == EnvelopeKind::AppendResponse
                        && envelope.meta().from == follower
                        && envelope.meta().to == leader
                        && envelope.meta().id != old
                })
                .last()
                .ok_or_else(|| ReplayFailure::Step {
                    step: engine.actions.len(),
                    message: "new AppendResp was not produced".to_owned(),
                })?;
            engine.deliver(newer)?;
            engine.checkpoint(Checkpoint::StaleAppendResponse { old, newer })?;
            engine.deliver(old)?;
            engine.release_all()?;
        }
        Template::DurableRestart => {
            let leader = engine.elect(candidate)?;
            engine.propose(leader, command(template, seed, 0))?;
            engine.release_all()?;
            let restart = if seed == SEEDS[0] {
                (leader + 1) % 3
            } else {
                (leader + 2) % 3
            };
            engine.restart(restart)?;
            engine.release_all()?;
        }
        Template::LaggingSnapshot => {
            let leader = engine.elect(candidate)?;
            let isolated = if seed == SEEDS[0] {
                (leader + 1) % 3
            } else {
                (leader + 2) % 3
            };
            let helper = (0..3)
                .find(|node| *node != leader && *node != isolated)
                .expect("three nodes");
            for serial in 0..4 {
                engine.propose(leader, command(template, seed, serial))?;
                engine.take_all()?;
                engine.deliver_matching(|envelope| {
                    envelope.meta().kind == EnvelopeKind::Append && envelope.meta().to == helper
                })?;
                engine.take_all()?;
                engine.deliver_matching(|envelope| {
                    envelope.meta().kind == EnvelopeKind::AppendResponse
                        && envelope.meta().from == helper
                        && envelope.meta().to == leader
                })?;
            }
            engine.compact(leader)?;
            for _ in 0..3 {
                engine.tick(leader)?;
            }
            engine.take_all()?;
            if engine
                .deliver_matching(|envelope| {
                    envelope.meta().kind == EnvelopeKind::InstallSnapshot
                        && envelope.meta().to == isolated
                })?
                .is_empty()
            {
                return Err(ReplayFailure::Safety(
                    SafetyViolation::SnapshotNotDelivered { node: isolated },
                ));
            }
            engine.checkpoint(Checkpoint::SnapshotDelivered { node: isolated })?;
            engine.release_all()?;
        }
        Template::ThreeToFourMembership | Template::FourToFiveMembership => {
            let leader = engine.elect(candidate)?;
            let newcomer = voters as NodeId;
            engine.add_learner(leader, newcomer)?;
            engine.release_all()?;
            for _ in 0..3 {
                engine.tick(leader)?;
                engine.release_all()?;
            }
            engine.promote(leader, newcomer)?;
            engine.take_all()?;
            let old: Vec<NodeId> = (0..voters as NodeId).collect();
            let old_supporters: Vec<_> = old
                .iter()
                .copied()
                .filter(|node| *node != leader)
                .take(voters / 2)
                .collect();
            // Every peer receives the promotion entry, but only an old majority
            // response reaches the leader. The leader then holds joint state.
            engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::Append
                    && (old.contains(&envelope.meta().to) || envelope.meta().to == newcomer)
            })?;
            engine.take_all()?;
            engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::AppendResponse
                    && envelope.meta().to == leader
                    && old_supporters.contains(&envelope.meta().from)
            })?;
            if !engine.hosts[engine.node_index(leader).expect("leader")]
                .view()
                .joint
            {
                return Err(ReplayFailure::Step {
                    step: engine.actions.len(),
                    message: "promotion did not enter joint state".to_owned(),
                });
            }
            let target_index = engine.hosts[engine.node_index(leader).expect("leader")]
                .view()
                .last_index;
            engine.take_all()?;
            let old_responses: BTreeSet<_> = engine
                .pending_ids(|envelope| {
                    envelope.meta().kind == EnvelopeKind::AppendResponse
                        && envelope.meta().to == leader
                })
                .into_iter()
                .collect();
            let partial = match template {
                Template::ThreeToFourMembership => vec![old_supporters[0]],
                Template::FourToFiveMembership => vec![old_supporters[0], newcomer],
                _ => unreachable!(),
            };
            let append_ids = engine.deliver_matching(|envelope| {
                envelope.meta().kind == EnvelopeKind::Append
                    && partial.contains(&envelope.meta().to)
            })?;
            if append_ids.len() != partial.len() {
                return Err(ReplayFailure::Step {
                    step: engine.actions.len(),
                    message: "one-sided leave append requests were not produced".to_owned(),
                });
            }
            engine.take_all()?;
            let responses = engine.pending_ids(|envelope| {
                envelope.meta().kind == EnvelopeKind::AppendResponse
                    && envelope.meta().to == leader
                    && !old_responses.contains(&envelope.meta().id)
                    && partial.contains(&envelope.meta().from)
            });
            if responses.len() != partial.len() {
                return Err(ReplayFailure::Step {
                    step: engine.actions.len(),
                    message: "one-sided leave responses absent".to_owned(),
                });
            }
            for response in &responses {
                engine.deliver(*response)?;
            }
            let mut acknowledged = vec![leader];
            acknowledged.extend(partial);
            engine.checkpoint(Checkpoint::JointOneSide {
                leader,
                outgoing: old,
                incoming: (0..=newcomer).collect(),
                acknowledged,
                target_index,
                append_ids,
                response_ids: responses,
            })?;
            engine.release_all()?;
            engine.tick(leader)?;
            engine.release_all()?;
        }
        Template::DemoteRemoveRecovery => {
            let leader = engine.elect(candidate)?;
            engine.propose(leader, command(template, seed, 0))?;
            engine.release_all()?;
            let wanted = if seed == SEEDS[0] { 1 } else { 0 };
            let victim = if wanted == leader { 2 } else { wanted };
            engine.demote(leader, victim)?;
            engine.release_all()?;
            engine.remove(leader, victim)?;
            engine.release_all()?;
            engine.propose(leader, command(template, seed, 1))?;
            engine.drain()?;
            let restart = (0..4)
                .find(|node| *node != leader && *node != victim)
                .expect("a surviving follower");
            engine.restart(restart)?;
            engine.release_all()?;
        }
    }
    engine.finish()?;
    Ok(engine.replay())
}

fn validate_replay(replay: &Replay) -> Result<(), ReplayFailure> {
    if replay.schema != TRACE_SCHEMA {
        return Err(ReplayFailure::Input("unknown trace schema".to_owned()));
    }
    let (voters, nodes) = replay.init.template.topology();
    if replay.init.voters != voters || replay.init.nodes != nodes || voters == 0 || nodes > 16 {
        return Err(ReplayFailure::Input("invalid trace init".to_owned()));
    }
    if !SEEDS.contains(&replay.init.seed) {
        return Err(ReplayFailure::Input("unknown trace seed".to_owned()));
    }
    if replay.actions.len() > MAX_ACTIONS {
        return Err(ReplayFailure::Input("trace action limit".to_owned()));
    }
    let known = |node: NodeId| node < replay.init.nodes as NodeId;
    let mut live = BTreeSet::new();
    let mut all_taken = BTreeSet::new();
    let mut restarts = 0;
    let mut compactions = 0;
    for action in &replay.actions {
        match action {
            Action::Tick { node }
            | Action::Propose { node, .. }
            | Action::AddLearner { node, .. }
            | Action::Promote { node, .. }
            | Action::Demote { node, .. }
            | Action::Remove { node, .. }
            | Action::Restart { node }
            | Action::Compact { node }
                if !known(*node) =>
            {
                return Err(ReplayFailure::Input("unknown node".to_owned()))
            }
            Action::Take { from, to, id, .. } => {
                if !known(*from) || !known(*to) {
                    return Err(ReplayFailure::Input("unknown node".to_owned()));
                }
                if !all_taken.insert(*id) || !live.insert(*id) {
                    return Err(ReplayFailure::Input(
                        "duplicate envelope reference".to_owned(),
                    ));
                }
            }
            Action::Deliver { id } if !live.remove(id) => {
                return Err(ReplayFailure::Input("bad envelope reference".to_owned()))
            }
            Action::AddLearner { peer, .. }
            | Action::Promote { peer, .. }
            | Action::Demote { peer, .. }
            | Action::Remove { peer, .. }
                if !known(*peer) =>
            {
                return Err(ReplayFailure::Input("unknown node".to_owned()))
            }
            Action::Restart { .. } => {
                restarts += 1;
                if restarts > MAX_RESTARTS {
                    return Err(ReplayFailure::Input("restart limit".to_owned()));
                }
            }
            Action::Compact { .. } => {
                compactions += 1;
                if compactions > MAX_COMPACTIONS {
                    return Err(ReplayFailure::Input("compaction limit".to_owned()));
                }
            }
            Action::Checkpoint { checkpoint } => match checkpoint {
                Checkpoint::ElectionSplit { first, second }
                    if !known(*first) || !known(*second) || first == second =>
                {
                    return Err(ReplayFailure::Input(
                        "invalid election split checkpoint".to_owned(),
                    ))
                }
                Checkpoint::StaleAppendResponse { old, newer } if old == newer => {
                    return Err(ReplayFailure::Input("invalid stale checkpoint".to_owned()))
                }
                Checkpoint::JointOneSide {
                    leader,
                    outgoing,
                    incoming,
                    acknowledged,
                    target_index,
                    append_ids,
                    response_ids,
                } if !known(*leader)
                    || outgoing.iter().any(|node| !known(*node))
                    || incoming.iter().any(|node| !known(*node))
                    || acknowledged.iter().any(|node| !known(*node))
                    || *target_index == 0
                    || append_ids.len() != response_ids.len()
                    || response_ids.len() + 1 != acknowledged.len() =>
                {
                    return Err(ReplayFailure::Input("unknown node".to_owned()));
                }
                Checkpoint::SnapshotDelivered { node } if !known(*node) => {
                    return Err(ReplayFailure::Input("unknown node".to_owned()))
                }
                _ => {}
            },
            _ => {}
        }
    }
    if !live.is_empty() {
        return Err(ReplayFailure::Input(
            "dangling envelope reference".to_owned(),
        ));
    }
    Ok(())
}

fn execute_trace(replay: &Replay, fault: Option<Mutant>) -> Result<(), ReplayFailure> {
    validate_replay(replay)?;
    let mut engine = Engine::new(replay.init.clone(), false, fault)?;
    for (step, action) in replay.actions.iter().cloned().enumerate() {
        let result = match action {
            Action::Tick { node } => engine.tick(node),
            Action::Take {
                from,
                to,
                id,
                kind,
                fingerprint,
            } => engine.replay_take(from, to, id, kind, fingerprint),
            Action::Deliver { id } => engine.deliver(id),
            Action::Propose { node, command } => engine.propose(node, command),
            Action::AddLearner { node, peer } => engine.add_learner(node, peer),
            Action::Promote { node, peer } => engine.promote(node, peer),
            Action::Demote { node, peer } => engine.demote(node, peer),
            Action::Remove { node, peer } => engine.remove(node, peer),
            Action::Restart { node } => engine.restart(node),
            Action::Compact { node } => engine.compact(node),
            Action::Checkpoint { checkpoint } => engine.checkpoint(checkpoint),
        };
        if let Err(error) = result {
            return Err(match error {
                ReplayFailure::Step { message, .. } => ReplayFailure::Step { step, message },
                other => other,
            });
        }
    }
    engine.finish()
}

fn load_replay(path: &std::path::Path) -> Result<Replay, ReplayFailure> {
    let bytes = std::fs::read(path)
        .map_err(|error| ReplayFailure::Input(format!("replay read: {error}")))?;
    if bytes.len() > REPLAY_MAX_BYTES {
        return Err(ReplayFailure::Input("replay exceeds 1 MiB".to_owned()));
    }
    let replay = serde_json::from_slice(&bytes)
        .map_err(|error| ReplayFailure::Input(format!("replay parse: {error}")))?;
    validate_replay(&replay)?;
    Ok(replay)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FailureSignature {
    Safety(SafetyViolation),
    Step(String),
}

fn failure_signature(failure: &ReplayFailure) -> Option<FailureSignature> {
    match failure {
        ReplayFailure::Safety(violation) => Some(FailureSignature::Safety(violation.clone())),
        ReplayFailure::Step { message, .. } => Some(FailureSignature::Step(message.clone())),
        ReplayFailure::Input(_) => None,
    }
}

fn matching_failure(replay: &Replay, fault: Option<Mutant>, expected: &FailureSignature) -> bool {
    execute_trace(replay, fault)
        .as_ref()
        .err()
        .and_then(failure_signature)
        .as_ref()
        == Some(expected)
}

struct DdminResult {
    replay: Replay,
    calls: usize,
    signature: FailureSignature,
}

/// It only accepts candidates that remain valid and reach the same executable
/// failure signature. Malformed envelope references never count as a reduction.
fn ddmin(mut replay: Replay, fault: Option<Mutant>, expected: FailureSignature) -> DdminResult {
    let mut granularity = 2usize;
    let mut calls = 0usize;
    while replay.actions.len() >= 2 && calls < MAX_DDMIN_REPLAYS {
        let chunk = replay.actions.len().div_ceil(granularity);
        let mut reduced = false;
        for start in (0..replay.actions.len()).step_by(chunk.max(1)) {
            if calls == MAX_DDMIN_REPLAYS {
                break;
            }
            let end = (start + chunk).min(replay.actions.len());
            let mut candidate = replay.clone();
            candidate.actions.drain(start..end);
            calls += 1;
            if validate_replay(&candidate).is_ok() && matching_failure(&candidate, fault, &expected)
            {
                replay = candidate;
                granularity = granularity.saturating_sub(1).max(2);
                reduced = true;
                break;
            }
        }
        if !reduced {
            if granularity >= replay.actions.len() {
                break;
            }
            granularity = (granularity * 2).min(replay.actions.len());
        }
    }
    assert!(calls <= MAX_DDMIN_REPLAYS, "ddmin replay limit");
    DdminResult {
        replay,
        calls,
        signature: expected,
    }
}

/// The environment entrypoint reduces executable safety and step failures. It
/// leaves parse and input failures alone because they do not describe a trace.
fn execute_requested_replay(replay: Replay) -> Result<Option<DdminResult>, ReplayFailure> {
    match execute_trace(&replay, None) {
        Ok(()) => Ok(None),
        Err(failure) => match failure_signature(&failure) {
            Some(signature) => Ok(Some(ddmin(replay, None, signature))),
            None => Err(failure),
        },
    }
}

fn minimized_replay_output(minimized: &DdminResult) -> String {
    serde_json::to_string(&minimized.replay).expect("minimized replay must serialize")
}

#[test]
fn fixed_adversarial_corpus_is_safe_and_seeded() {
    for template in Template::ALL {
        let first = build_trace(template, SEEDS[0], None)
            .unwrap_or_else(|error| panic!("{template:?} seed 1: {error}"));
        let second = build_trace(template, SEEDS[1], None)
            .unwrap_or_else(|error| panic!("{template:?} seed 2: {error}"));
        assert!(first.actions.len() <= MAX_ACTIONS);
        assert!(second.actions.len() <= MAX_ACTIONS);
        let first_digest = Sha256::digest(serde_json::to_vec(&first).expect("serialize trace"));
        let second_digest = Sha256::digest(serde_json::to_vec(&second).expect("serialize trace"));
        assert_ne!(
            first_digest[..],
            second_digest[..],
            "{template:?} ignores the seed"
        );
    }
}

#[test]
fn every_generated_trace_serializes_parses_and_replays_fresh() {
    for template in Template::ALL {
        for seed in SEEDS {
            let trace = build_trace(template, seed, None)
                .unwrap_or_else(|error| panic!("build {template:?} {seed:#x}: {error}"));
            let bytes = serde_json::to_vec(&trace).expect("serialize trace");
            let parsed: Replay = serde_json::from_slice(&bytes).expect("parse trace");
            execute_trace(&parsed, None)
                .unwrap_or_else(|error| panic!("replay {template:?} {seed:#x}: {error}"));
        }
    }
}

#[test]
fn replay_input_is_strict_bounded_and_optional() {
    if let Some(path) = std::env::var_os("RAFT_RUNTIME_ADVERSARIAL_RECOVERY_REPLAY") {
        let replay =
            load_replay(std::path::Path::new(&path)).expect("requested replay input must be valid");
        match execute_requested_replay(replay) {
            Ok(None) => {}
            Ok(Some(minimized)) => {
                eprintln!("{}", minimized_replay_output(&minimized));
                panic!(
                    "requested replay has executable failure {:?} after {} ddmin calls",
                    minimized.signature, minimized.calls
                );
            }
            Err(error) => panic!("requested replay execution failed: {error}"),
        }
    }
    let directory = TempDir::new().expect("temp input directory");
    assert!(matches!(
        load_replay(&directory.path().join("missing.json")),
        Err(ReplayFailure::Input(_))
    ));
    let large = directory.path().join("large.json");
    std::fs::write(&large, vec![b'x'; REPLAY_MAX_BYTES + 1]).expect("write large input");
    assert!(matches!(load_replay(&large), Err(ReplayFailure::Input(_))));
    for invalid in [
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"extra":1,"actions":[]}"#,
        r#"{"schema":"wrong","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[{"action":"unknown"}]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[{"action":"tick","node":9}]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[{"action":"take","from":0,"to":1,"id":1,"kind":"vote","fingerprint":"x"},{"action":"take","from":0,"to":1,"id":1,"kind":"vote","fingerprint":"x"}]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[{"action":"deliver","id":1}]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":1},"actions":[{"action":"take","from":0,"to":1,"id":1,"kind":"vote","fingerprint":"x"}]}"#,
        r#"{"schema":"raft-runtime/adversarial-recovery/v1","init":{"voters":3,"nodes":3,"template":"election-split","seed":2},"actions":[]}"#,
    ] {
        let path = directory
            .path()
            .join(format!("{:x}.json", Sha256::digest(invalid.as_bytes())));
        std::fs::write(&path, invalid).expect("write invalid input");
        assert!(
            matches!(load_replay(&path), Err(ReplayFailure::Input(_))),
            "{invalid}"
        );
    }
}

#[test]
fn requested_safety_replay_uses_the_same_signature_reducer() {
    let replay = Replay {
        schema: TRACE_SCHEMA.to_owned(),
        init: TraceInit {
            voters: 3,
            nodes: 3,
            template: Template::ElectionSplit,
            seed: SEEDS[0],
        },
        actions: vec![Action::Checkpoint {
            checkpoint: Checkpoint::SnapshotDelivered { node: 0 },
        }],
    };
    let minimized = execute_requested_replay(replay)
        .expect("the structurally valid request reaches a safety failure")
        .expect("safety failures request minimization");
    assert!(minimized.calls <= MAX_DDMIN_REPLAYS);
    let output = minimized_replay_output(&minimized);
    let printed: Replay = serde_json::from_str(&output).expect("stderr payload is JSON");
    assert_eq!(printed.schema, TRACE_SCHEMA);
    let bytes = serde_json::to_vec(&minimized.replay).expect("minimized JSON");
    let parsed: Replay = serde_json::from_slice(&bytes).expect("minimized replay parses");
    assert_eq!(
        minimized.signature,
        FailureSignature::Safety(SafetyViolation::SnapshotNotDelivered { node: 0 })
    );
    assert!(matching_failure(&parsed, None, &minimized.signature));
}

#[test]
fn requested_step_replay_uses_the_same_message_signature_reducer() {
    let replay = Replay {
        schema: TRACE_SCHEMA.to_owned(),
        init: TraceInit {
            voters: 3,
            nodes: 3,
            template: Template::ElectionSplit,
            seed: SEEDS[0],
        },
        actions: vec![Action::Checkpoint {
            checkpoint: Checkpoint::ElectionSplit {
                first: 0,
                second: 1,
            },
        }],
    };
    let minimized = execute_requested_replay(replay)
        .expect("the structurally valid request reaches a step failure")
        .expect("step failures request minimization");
    assert!(minimized.calls <= MAX_DDMIN_REPLAYS);
    assert!(matches!(
        &minimized.signature,
        FailureSignature::Step(message)
            if message == "election split window is not two candidates without a leader"
    ));
    let output = minimized_replay_output(&minimized);
    let parsed: Replay = serde_json::from_str(&output).expect("stderr payload is JSON");
    validate_replay(&parsed).expect("minimized step replay remains structural");
    assert!(matching_failure(&parsed, None, &minimized.signature));
}

#[test]
fn runtime_rejects_structurally_valid_but_forged_take_metadata() {
    let trace = build_trace(Template::ElectionSplit, SEEDS[0], None).expect("election trace");

    let mut wrong_fingerprint = trace.clone();
    let take = wrong_fingerprint
        .actions
        .iter_mut()
        .find_map(|action| match action {
            Action::Take { fingerprint, .. } => Some(fingerprint),
            _ => None,
        })
        .expect("trace has a take");
    *take = "forged-fingerprint".to_owned();
    validate_replay(&wrong_fingerprint).expect("shape remains valid");
    assert!(matches!(
        execute_trace(&wrong_fingerprint, None),
        Err(ReplayFailure::Step { .. })
    ));

    let mut wrong_id = trace;
    let old_id = wrong_id
        .actions
        .iter()
        .find_map(|action| match action {
            Action::Take { id, .. } => Some(*id),
            _ => None,
        })
        .expect("trace has a take");
    let forged_id = u64::MAX - 7;
    for action in &mut wrong_id.actions {
        match action {
            Action::Take { id, .. } | Action::Deliver { id } if *id == old_id => {
                *id = forged_id;
            }
            _ => {}
        }
    }
    validate_replay(&wrong_id).expect("references remain structural");
    assert!(matches!(
        execute_trace(&wrong_id, None),
        Err(ReplayFailure::Step { .. })
    ));
}

#[test]
fn action_limit_refuses_before_the_193rd_host_mutation() {
    let init = TraceInit {
        voters: 3,
        nodes: 3,
        template: Template::ElectionSplit,
        seed: SEEDS[0],
    };
    let mut engine = Engine::new(init, true, None).expect("engine");
    for _ in 0..49 {
        engine.tick(0).expect("preparation tick");
    }
    engine.actions = vec![Action::Tick { node: 0 }; MAX_ACTIONS];
    let before = engine.hosts[0].view();
    assert!(matches!(engine.tick(0), Err(ReplayFailure::Step { .. })));
    assert_eq!(
        engine.hosts[0].view(),
        before,
        "the rejected tick did not start an election"
    );
}

#[test]
fn shared_executor_invariants_kill_all_three_mutants() {
    for (template, mutant) in [
        (
            Template::StaleAppendResp,
            Mutant::RegressOnlyStaleAppendResponse,
        ),
        (Template::ThreeToFourMembership, Mutant::OneSidedJointQuorum),
        (Template::FourToFiveMembership, Mutant::OneSidedJointQuorum),
        (Template::DurableRestart, Mutant::IgnorePersistedMembership),
    ] {
        let trace = build_trace(template, SEEDS[0], None).expect("baseline trace");
        assert!(execute_trace(&trace, None).is_ok(), "baseline {template:?}");
        assert!(
            matches!(
                execute_trace(&trace, Some(mutant)),
                Err(ReplayFailure::Safety(_))
            ),
            "{template:?} missed {mutant:?}"
        );
    }
}

#[test]
fn ddmin_keeps_the_same_safety_signature_and_valid_json() {
    let trace = build_trace(Template::StaleAppendResp, SEEDS[0], None).expect("stale trace");
    let expected = match execute_trace(&trace, Some(Mutant::RegressOnlyStaleAppendResponse)) {
        Err(ReplayFailure::Safety(violation)) => FailureSignature::Safety(violation),
        other => panic!("stale mutant must fail with safety, got {other:?}"),
    };
    let minimized = ddmin(
        trace,
        Some(Mutant::RegressOnlyStaleAppendResponse),
        expected.clone(),
    );
    assert!(minimized.calls <= MAX_DDMIN_REPLAYS);
    assert_eq!(minimized.signature, expected);
    let json = serde_json::to_vec(&minimized.replay).expect("serialize minimized trace");
    let parsed: Replay = serde_json::from_slice(&json).expect("parse minimized trace");
    validate_replay(&parsed).expect("minimized trace remains structural");
    assert!(matching_failure(
        &parsed,
        Some(Mutant::RegressOnlyStaleAppendResponse),
        &expected
    ));
}
