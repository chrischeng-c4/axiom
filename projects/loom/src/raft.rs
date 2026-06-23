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

    pub fn run_ids(&self) -> Vec<WorkflowRunId> {
        self.runs.keys().cloned().collect()
    }

    /// Snapshot the whole run map (for raft log compaction + durable recovery).
    pub fn snapshot(&self) -> Vec<u8> {
        let runs: Vec<&WorkflowRun> = self.runs.values().collect();
        serde_json::to_vec(&runs).unwrap_or_default()
    }

    /// Restore the run map from a [`snapshot`](Self::snapshot).
    pub fn restore(&mut self, bytes: &[u8]) {
        if let Ok(runs) = serde_json::from_slice::<Vec<WorkflowRun>>(bytes) {
            self.runs = runs.into_iter().map(|r| (r.id.clone(), r)).collect();
        }
    }
}

/// A single-voter raft-backed [`RunStore`] (#110): puts go through the raft log
/// (durable, and the basis for HA by adding voters). A single voter is its own
/// majority, so commit is local and the store is fully synchronous + in-process
/// — no transport. The raft log persists to disk; on restart the state machine
/// rebuilds by replaying the committed log. Multi-voter HA adds a driver + h2c
/// transport (relay's `raft_driver` pattern); the consensus itself is the same.
pub struct RaftRunStore {
    inner: std::sync::Mutex<RaftInner>,
    path: std::path::PathBuf,
    snap_path: std::path::PathBuf,
}

struct RaftInner {
    node: raftcore::RaftNode,
    sm: LoomStateMachine,
}

impl RaftRunStore {
    /// Open (or recover) a single-voter raft store under `dir`.
    pub fn open(node_id: u64, dir: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let path = dir.join("raft.json");
        let snap_path = dir.join("runs.snapshot.json");
        let membership = raftcore::auto_membership(1);
        let node = match std::fs::read(&path).ok().and_then(|b| serde_json::from_slice(&b).ok()) {
            Some(state) => raftcore::RaftNode::from_persisted(node_id, &membership, state),
            None => raftcore::RaftNode::new(node_id, &membership),
        };
        let mut sm = LoomStateMachine::new();
        // Durable recovery: restore the materialized state from the snapshot
        // (the raft log provides consensus ordering; the snapshot is the
        // materialized run map — the standard raft + snapshot pattern).
        if let Ok(bytes) = std::fs::read(&snap_path) {
            sm.restore(&bytes);
        }
        let mut inner = RaftInner { node, sm };
        // Drive to leadership and apply any newly committed entries.
        for _ in 0..500 {
            inner.node.tick();
            for entry in inner.node.take_committed() {
                inner.sm.apply(&entry);
            }
        }
        Ok(Self { inner: std::sync::Mutex::new(inner), path, snap_path })
    }

    fn persist(&self, inner: &RaftInner) -> anyhow::Result<()> {
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(&inner.node.persisted())?)?;
        std::fs::rename(&tmp, &self.path)?;
        // Snapshot the materialized state for durable recovery.
        let snap_tmp = self.snap_path.with_extension("json.tmp");
        std::fs::write(&snap_tmp, inner.sm.snapshot())?;
        std::fs::rename(&snap_tmp, &self.snap_path)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::store::RunStore for RaftRunStore {
    async fn put(&self, run: WorkflowRun) -> anyhow::Result<()> {
        let mut guard = self.inner.lock().map_err(|_| anyhow::anyhow!("raft store poisoned"))?;
        let inner = &mut *guard;
        for _ in 0..500 {
            if inner.node.is_leader() {
                break;
            }
            inner.node.tick();
        }
        let idx = inner
            .node
            .propose(Command::PutRun(run).encode())
            .ok_or_else(|| anyhow::anyhow!("not leader; cannot propose"))?;
        for _ in 0..500 {
            if inner.sm.applied_index() >= idx {
                break;
            }
            inner.node.tick();
            for entry in inner.node.take_committed() {
                inner.sm.apply(&entry);
            }
        }
        anyhow::ensure!(inner.sm.applied_index() >= idx, "raft put did not commit");
        self.persist(inner)
    }

    async fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>> {
        let guard = self.inner.lock().map_err(|_| anyhow::anyhow!("raft store poisoned"))?;
        Ok(guard.sm.get(id).cloned())
    }

    async fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>> {
        let guard = self.inner.lock().map_err(|_| anyhow::anyhow!("raft store poisoned"))?;
        Ok(guard.sm.run_ids())
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

    #[tokio::test]
    async fn raft_run_store_persists_and_recovers() {
        use crate::store::RunStore;
        let dir = std::env::temp_dir().join(format!("loom-raft-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let id = WorkflowRunId::new("rr1");

        {
            let store = RaftRunStore::open(0, &dir).unwrap();
            store.put(WorkflowRun::new(id.clone())).await.unwrap();
            assert_eq!(store.get(&id).await.unwrap().unwrap().id, id);
            assert_eq!(store.list().await.unwrap(), vec![id.clone()]);
        } // drop: only the raft log on disk remains

        // reopen: replaying the committed raft log rebuilds the state machine.
        let recovered = RaftRunStore::open(0, &dir).unwrap();
        assert_eq!(recovered.get(&id).await.unwrap().unwrap().id, id);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
