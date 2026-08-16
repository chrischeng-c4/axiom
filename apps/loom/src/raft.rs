//! Raft-replicated workflow state (#110) — loom's `RunStore` over the shared
//! [`raft_host`] driver.
//!
//! loom's durable state is its run map. A raft group replicates it: each `put`
//! is a [`Command::PutRun`] proposed to the leader via [`raft_host::RaftHost`];
//! `propose` returns only after the local state machine applies it
//! (read-your-write), so a subsequent `get` sees it. Every replica converges.
//!
//! [`raft_host`] owns the tick/pump loop, the h2c peer transport (Vote /
//! Append / InstallSnapshot), the single apply loop, and snapshot + log
//! compaction — loom supplies only the [`LoomSm`] state machine
//! (`apply`/`snapshot`/`restore`/`applied_index`) and gets HA + the backup
//! layer for free. This converged loom's two hand-rolled raft stacks into one
//! and wired install-snapshot (#546).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use serde::{Deserialize, Serialize};

use crate::model::{WorkflowRun, WorkflowRunId};

/// A replicated state-machine command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    PutRun(WorkflowRun),
    /// Remove a run from the replicated map (completed-DAG GC, #106).
    DeleteRun(WorkflowRunId),
}

impl Command {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("encode raft command")
    }
    pub fn decode(bytes: &[u8]) -> anyhow::Result<Command> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

/// The materialized snapshot wire/disk form: the run map + the log index it
/// covers. The `up_to` index travels with the map so install-snapshot (follower
/// catch-up) and cold-start recovery both set the applied index correctly.
#[derive(Serialize, Deserialize, Default)]
struct LoomSnapshot {
    up_to: u64,
    runs: Vec<WorkflowRun>,
}

/// loom's [`raft_runtime::RaftStateMachine`]: the replicated run map. `apply` folds
/// committed [`Command`]s into the map; `snapshot`/`restore` carry the map +
/// applied index (for compaction, install-snapshot, and durable recovery).
///
/// Unlike a memory-only SM, [`LoomSm`] persists its materialized snapshot to
/// disk on every apply (`snap_path`), so a cold start recovers the run map
/// without replaying the whole log or waiting for a fresh-term commit.
pub struct LoomSm {
    runs: Mutex<BTreeMap<WorkflowRunId, WorkflowRun>>,
    applied: AtomicU64,
    snap_path: Option<std::path::PathBuf>,
}

impl LoomSm {
    /// Build the state machine, restoring the on-disk materialized snapshot when
    /// `snap_path` is set and present (durable recovery). Pass `None` for an
    /// in-process (test) SM with no disk persistence.
    pub fn new(snap_path: Option<std::path::PathBuf>) -> Arc<Self> {
        let sm = LoomSm {
            runs: Mutex::new(BTreeMap::new()),
            applied: AtomicU64::new(0),
            snap_path,
        };
        if let Some(p) = &sm.snap_path {
            if let Ok(bytes) = std::fs::read(p) {
                sm.load_snapshot(&bytes);
            }
        }
        Arc::new(sm)
    }

    fn load_snapshot(&self, bytes: &[u8]) {
        if let Ok(snap) = serde_json::from_slice::<LoomSnapshot>(bytes) {
            *self.runs.lock().unwrap() =
                snap.runs.into_iter().map(|r| (r.id.clone(), r)).collect();
            self.applied.store(snap.up_to, Ordering::Release);
        }
    }

    fn encode_snapshot(&self) -> Vec<u8> {
        let runs = self.runs.lock().unwrap();
        let snap = LoomSnapshot {
            up_to: self.applied.load(Ordering::Acquire),
            runs: runs.values().cloned().collect(),
        };
        serde_json::to_vec(&snap).unwrap_or_default()
    }

    /// Atomically write the materialized snapshot to `snap_path` (temp + rename).
    fn persist_snapshot(&self) {
        let Some(p) = &self.snap_path else { return };
        let bytes = self.encode_snapshot();
        let tmp = p.with_extension("json.tmp");
        if std::fs::write(&tmp, &bytes).is_ok() {
            let _ = std::fs::rename(&tmp, p);
        }
    }

    /// Read a run from the applied map (local read — no raft round-trip).
    pub fn get(&self, id: &WorkflowRunId) -> Option<WorkflowRun> {
        self.runs.lock().unwrap().get(id).cloned()
    }
    /// The ids of all applied runs (ordered).
    pub fn run_ids(&self) -> Vec<WorkflowRunId> {
        self.runs.lock().unwrap().keys().cloned().collect()
    }
    /// Applied run count.
    pub fn len(&self) -> usize {
        self.runs.lock().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl raft_runtime::RaftStateMachine for LoomSm {
    fn apply(&self, index: raft_runtime::Index, command: &[u8]) -> anyhow::Result<()> {
        // The leader's blank term-marker entry (empty command) is a no-op.
        if !command.is_empty() {
            match Command::decode(command) {
                Ok(Command::PutRun(run)) => {
                    self.runs.lock().unwrap().insert(run.id.clone(), run);
                }
                Ok(Command::DeleteRun(id)) => {
                    self.runs.lock().unwrap().remove(&id);
                }
                // A malformed command must not stall the apply loop.
                Err(_) => {}
            }
        }
        self.applied.store(index, Ordering::Release);
        self.persist_snapshot();
        Ok(())
    }

    fn snapshot(&self, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        let bytes = self.encode_snapshot();
        writer.write_all(&bytes)?;
        Ok(())
    }

    fn restore(&self, reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        self.load_snapshot(&bytes);
        self.persist_snapshot();
        Ok(())
    }

    fn applied_index(&self) -> raft_runtime::Index {
        self.applied.load(Ordering::Acquire)
    }
}

/// A raft-replicated [`RunStore`](crate::store::RunStore) over the shared
/// [`raft_runtime::RaftHost`]. Writes (`put`/`delete`) propose a [`Command`] and
/// return after the local SM applies it (read-your-write); reads serve from the
/// applied map. Single-node by default (its own majority, no transport); pass
/// peers for a multi-voter HA group. Peer transport, snapshot, and log
/// compaction come from the host.
pub struct RaftRunStore {
    host: Arc<raft_runtime::RaftHost>,
    sm: Arc<LoomSm>,
}

impl RaftRunStore {
    /// Build a raft store for node `id` with `membership` + `peers`, persisting
    /// the raft log and the materialized snapshot under `dir`.
    pub fn new(
        id: raft_runtime::NodeId,
        membership: raft_runtime::Membership,
        peers: HashMap<raft_runtime::NodeId, String>,
        dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;
        let dir_str = dir
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("raft dir path is not valid UTF-8"))?;
        let sm = LoomSm::new(Some(dir.join("runs.snapshot.json")));
        let store = raft_runtime::RaftStore::open(dir_str, id, raft_runtime::FsyncPolicy::Always)?;
        let host = raft_runtime::RaftHost::spawn(
            id,
            membership,
            peers,
            store,
            sm.clone() as Arc<dyn raft_runtime::RaftStateMachine>,
            raft_runtime::HostConfig::default(),
        );
        Ok(Self { host: Arc::new(host), sm })
    }

    /// Single-node raft store: node 0 is its own majority, no peers. The archetype
    /// default for local/dev; k8s flips to replica mode on scale-out.
    pub fn single_node(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::new(
            0,
            raft_runtime::Membership { voters: vec![0], learners: vec![] },
            HashMap::new(),
            dir,
        )
    }

    /// Multi-voter cluster from an explicit peer map (local multi-node testing,
    /// `LOOM_CLUSTER_PEERS`): voters `0..n_voters`.
    pub fn cluster(
        id: raft_runtime::NodeId,
        n_voters: u64,
        peers: HashMap<raft_runtime::NodeId, String>,
        dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        Self::new(id, raft_runtime::auto_membership(n_voters), peers, dir)
    }

    /// k8s auto-mode: derive node id / membership / peers from the StatefulSet
    /// downward API via [`raft_runtime::ClusterTopology`].
    pub fn from_topology(
        topo: raft_runtime::ClusterTopology,
        dir: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        Self::new(topo.node_id, topo.membership, topo.peers, dir)
    }

    /// The peer-transport + `/raftz` router to merge into the controller's h2c
    /// port (Vote / Append / InstallSnapshot / publish).
    pub fn router(&self) -> Router {
        self.host.router()
    }

    async fn propose(&self, cmd: Command) -> anyhow::Result<()> {
        self.host.propose(cmd.encode()).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::store::RunStore for RaftRunStore {
    async fn put(&self, run: WorkflowRun) -> anyhow::Result<()> {
        self.propose(Command::PutRun(run)).await
    }
    async fn delete(&self, id: &WorkflowRunId) -> anyhow::Result<()> {
        self.propose(Command::DeleteRun(id.clone())).await
    }
    async fn get(&self, id: &WorkflowRunId) -> anyhow::Result<Option<WorkflowRun>> {
        Ok(self.sm.get(id))
    }
    async fn list(&self) -> anyhow::Result<Vec<WorkflowRunId>> {
        Ok(self.sm.run_ids())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raft_core::{auto_membership, NodeId, RaftEntry, RaftNode};
    // Bring the SM trait into scope so the in-process cluster can call `apply`.
    use raft_runtime::RaftStateMachine as _;

    /// An in-process raft cluster with a [`LoomSm`] per node — proves loom's
    /// command semantics replicate + converge under `raft_core` consensus.
    struct Cluster {
        nodes: BTreeMap<NodeId, RaftNode>,
        sms: BTreeMap<NodeId, Arc<LoomSm>>,
    }

    impl Cluster {
        fn new(n: u64) -> Self {
            let m = auto_membership(n);
            Self {
                nodes: (0..n).map(|i| (i, RaftNode::new(i, &m))).collect(),
                sms: (0..n).map(|i| (i, LoomSm::new(None))).collect(),
            }
        }

        fn step(&mut self) {
            for node in self.nodes.values_mut() {
                node.tick();
            }
            // route outgoing messages
            let mut bus: Vec<(NodeId, raft_core::Outgoing)> = Vec::new();
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
                let sm = self.sms.get(&id).unwrap();
                for entry in &entries {
                    let _ = sm.apply(entry.index, &entry.command);
                }
            }
        }

        fn leader(&self) -> Option<NodeId> {
            self.nodes
                .iter()
                .find(|(_, n)| n.is_leader())
                .map(|(id, _)| *id)
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

        fn run_until_leader_count(&mut self) -> usize {
            self.run_until_leader();
            self.nodes.values().filter(|n| n.is_leader()).count()
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
            c.propose(Command::PutRun(WorkflowRun::new(WorkflowRunId::new(
                format!("r{i}"),
            ))));
        }
        // a few more steps to flush replication to every follower
        for _ in 0..30 {
            c.step();
        }
        for sm in c.sms.values() {
            assert_eq!(sm.len(), 3);
        }
    }

    #[tokio::test]
    async fn raft_run_store_persists_and_recovers() {
        use crate::store::RunStore;
        let dir = std::env::temp_dir().join(format!("loom-raft-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let id = WorkflowRunId::new("rr1");

        {
            let store = RaftRunStore::single_node(&dir).unwrap();
            store.put(WorkflowRun::new(id.clone())).await.unwrap();
            assert_eq!(store.get(&id).await.unwrap().unwrap().id, id);
            assert_eq!(store.list().await.unwrap(), vec![id.clone()]);
        } // drop: the raft log + materialized snapshot on disk remain

        // reopen: the materialized snapshot restores the run map (no write needed).
        let recovered = RaftRunStore::single_node(&dir).unwrap();
        assert_eq!(recovered.get(&id).await.unwrap().unwrap().id, id);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
