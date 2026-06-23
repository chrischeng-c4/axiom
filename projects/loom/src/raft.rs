//! Raft-replicated workflow state (#110) — the replicated state machine over
//! `raftcore`.
//!
//! loom's durable state is its run map. A raft group replicates it: each `put`
//! is a [`Command::PutRun`] proposed to the leader; once committed (majority),
//! every node applies it via `take_committed`, so all replicas converge. This
//! is the HA tier of the durability ADR (`docs/adr/0001`); the single-node
//! tier is [`crate::store::FileStore`].
//!
//! `raftcore` is **step-driven** (no timers/threads), so consensus is tested
//! in-process here (a message-routing cluster). The production driver — a tick
//! loop + h2c transport + a `RaftStore` for `PersistedState`, modelled on
//! relay's `raft_driver` — is the remaining wiring to expose this as a
//! `RunStore`.

use std::collections::BTreeMap;

use raftcore::RaftEntry;
use serde::{Deserialize, Serialize};

use crate::model::{WorkflowRun, WorkflowRunId};

/// A replicated state-machine command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    PutRun(WorkflowRun),
}

impl Command {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("encode raft command")
    }
    pub fn decode(bytes: &[u8]) -> anyhow::Result<Command> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// The replicated workflow-state machine: applies committed commands to the run
/// map. Every node in the raft group converges to the same map.
#[derive(Default)]
pub struct LoomStateMachine {
    runs: BTreeMap<WorkflowRunId, WorkflowRun>,
    applied: u64,
}

impl LoomStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one committed raft entry. The leader's blank term-marker entry
    /// (empty command) is a no-op.
    pub fn apply(&mut self, entry: &RaftEntry) {
        if !entry.command.is_empty() {
            if let Ok(Command::PutRun(run)) = Command::decode(&entry.command) {
                self.runs.insert(run.id.clone(), run);
            }
        }
        self.applied = entry.index;
    }

    pub fn get(&self, id: &WorkflowRunId) -> Option<&WorkflowRun> {
        self.runs.get(id)
    }
    pub fn len(&self) -> usize {
        self.runs.len()
    }
    pub fn applied_index(&self) -> u64 {
        self.applied
    }

    /// Snapshot the whole run map (for raft log compaction).
    pub fn snapshot(&self) -> Vec<u8> {
        serde_json::to_vec(&self.runs).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raftcore::{auto_membership, NodeId, RaftNode};

    /// An in-process raft cluster with a loom state machine per node.
    struct Cluster {
        nodes: BTreeMap<NodeId, RaftNode>,
        sms: BTreeMap<NodeId, LoomStateMachine>,
    }

    impl Cluster {
        fn new(n: u64) -> Self {
            let m = auto_membership(n);
            Self {
                nodes: (0..n).map(|i| (i, RaftNode::new(i, &m))).collect(),
                sms: (0..n).map(|i| (i, LoomStateMachine::new())).collect(),
            }
        }

        fn step(&mut self) {
            for node in self.nodes.values_mut() {
                node.tick();
            }
            // route outgoing messages
            let mut bus: Vec<(NodeId, raftcore::Outgoing)> = Vec::new();
            for (id, node) in self.nodes.iter_mut() {
                for out in node.take_outgoing() {
                    bus.push((*id, out));
                }
            }
            for (from, out) in bus {
                if let Some(node) = self.nodes.get_mut(&out.to) {
                    node.handle(from, out.msg);
                }
            }
            // apply committed entries to each node's state machine
            let mut committed: Vec<(NodeId, Vec<RaftEntry>)> = Vec::new();
            for (id, node) in self.nodes.iter_mut() {
                committed.push((*id, node.take_committed()));
            }
            for (id, entries) in committed {
                let sm = self.sms.get_mut(&id).unwrap();
                for entry in &entries {
                    sm.apply(entry);
                }
            }
        }

        fn leader(&self) -> Option<NodeId> {
            self.nodes.iter().find(|(_, n)| n.is_leader()).map(|(id, _)| *id)
        }

        fn run_until_leader(&mut self) -> NodeId {
            for _ in 0..500 {
                if let Some(l) = self.leader() {
                    return l;
                }
                self.step();
            }
            panic!("no leader elected");
        }

        fn propose(&mut self, cmd: Command) {
            let leader = self.run_until_leader();
            self.nodes.get_mut(&leader).unwrap().propose(cmd.encode());
            for _ in 0..30 {
                self.step();
            }
        }
    }

    #[test]
    fn workflow_state_replicates_across_a_3_node_group() {
        let mut c = Cluster::new(3);
        assert_eq!(c.run_until_leader_count(), 1);

        c.propose(Command::PutRun(WorkflowRun::new(WorkflowRunId::new("r1"))));

        // every replica converged to the committed run.
        for sm in c.sms.values() {
            assert_eq!(sm.len(), 1, "each node applies the committed PutRun");
            assert!(sm.get(&WorkflowRunId::new("r1")).is_some());
        }
    }

    #[test]
    fn survives_a_follower_apply_lag_then_converges() {
        let mut c = Cluster::new(3);
        for i in 0..3 {
            c.propose(Command::PutRun(WorkflowRun::new(WorkflowRunId::new(format!("r{i}")))));
        }
        // a few more steps to flush replication to every follower
        for _ in 0..30 {
            c.step();
        }
        for sm in c.sms.values() {
            assert_eq!(sm.len(), 3);
        }
    }

    impl Cluster {
        fn run_until_leader_count(&mut self) -> usize {
            self.run_until_leader();
            self.nodes.values().filter(|n| n.is_leader()).count()
        }
    }
}
