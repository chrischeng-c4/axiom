//! raft_core-backed consensus (HA phase C). keep's engine is wired as a Raft
//! state machine so writes go through the log (propose → commit → apply) using
//! the shared [`raft_core`] crate (`libs/raft-core`) — the same verified engine
//! relay uses, replacing the earlier openraft integration.
//!
//! - [`RaftKv`] is **one** Raft group fronting the engine. For a single node it
//!   is a sole voter that commits locally; the command is a [`WalOp`] (the same
//!   type the WAL + recovery already use) and apply reuses the WAL-replay path.
//! - [`ShardedRaft`] runs **one group per owned shard** (HA phase A's keyspace
//!   split, now each shard independently replicated), routing a write to its
//!   key's shard.
//!
//! Multi-node networking (an h2c driver feeding `handle`/`take_outgoing` across
//! pods, mirroring relay's driver) is the remaining slice; the consensus core,
//! per-shard structure, snapshot/compaction and apply path are all here. See
//! HA.md.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use raft_core::{Membership, NodeId, RaftNode};
use serde::{Deserialize, Serialize};

use crate::cluster::ClusterConfig;
use crate::engine::KvEngine;
use crate::persistence::format::WalOp;
use crate::persistence::recovery::RecoveryManager;
use crate::types::KvValue;

/// App response for a committed command (the engine result isn't threaded back —
/// the command's own HTTP handler already has it).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Response {
    pub applied: bool,
}

/// A one-member voter set for a single-node group.
fn solo(node_id: NodeId) -> Membership {
    Membership {
        voters: vec![node_id],
        learners: vec![],
    }
}

/// Tick a node until it wins its (sole-voter) election.
fn drive_to_leader(node: &mut RaftNode) {
    for _ in 0..1000 {
        if node.is_leader() {
            return;
        }
        node.tick();
        let _ = node.take_outgoing();
    }
}

/// Apply any received snapshot, then newly committed commands, to the engine.
fn apply(node: &mut RaftNode, engine: &KvEngine) {
    if let Some(snap) = node.take_installed_snapshot() {
        if let Ok(dump) = serde_json::from_slice::<Vec<(String, KvValue)>>(&snap) {
            engine.load_values(dump);
        }
    }
    for e in node.take_committed() {
        if let Ok(op) = serde_json::from_slice::<WalOp>(&e.command) {
            // Reuse the exact WAL-replay apply path.
            let _ = RecoveryManager::apply_one(engine, &op);
        }
    }
}

/// A single raft_core group fronting a keep engine; writes go through consensus.
pub struct RaftKv {
    node: Mutex<RaftNode>,
    pub engine: Arc<KvEngine>,
}

impl RaftKv {
    /// Build a single-node group and elect it (a one-member cluster).
    pub async fn single_node(node_id: NodeId, engine: Arc<KvEngine>) -> anyhow::Result<Self> {
        let mut node = RaftNode::new(node_id, &solo(node_id));
        drive_to_leader(&mut node);
        if !node.is_leader() {
            anyhow::bail!("single-node group failed to elect a leader");
        }
        Ok(Self {
            node: Mutex::new(node),
            engine,
        })
    }

    /// Propose a mutation through Raft; resolves once committed + applied.
    pub async fn write(&self, op: WalOp) -> anyhow::Result<Response> {
        let bytes = serde_json::to_vec(&op)?;
        let mut node = self.node.lock();
        let idx = node
            .propose(bytes)
            .ok_or_else(|| anyhow::anyhow!("not the leader"))?;
        // A sole voter commits immediately; apply what committed.
        apply(&mut node, &self.engine);
        Ok(Response {
            applied: node.commit_index() >= idx,
        })
    }

    /// Snapshot the engine into the Raft log, compacting entries up to the
    /// commit point (so a lagging/new replica is shipped state, not full history).
    pub async fn snapshot(&self) -> anyhow::Result<()> {
        let dump = self.engine.dump_values();
        let data = serde_json::to_vec(&dump)?;
        let mut node = self.node.lock();
        let up_to = node.commit_index();
        node.compact(up_to, data);
        Ok(())
    }

    pub fn is_leader(&self) -> bool {
        self.node.lock().is_leader()
    }
}

/// One Raft group per owned shard; a write routes to its key's shard.
pub struct ShardedRaft {
    cluster: ClusterConfig,
    groups: HashMap<u32, RaftKv>,
    pub engine: Arc<KvEngine>,
}

impl ShardedRaft {
    /// Spin up one group per shard this node owns (`cluster.owned_shards()`).
    pub async fn new(cluster: ClusterConfig, engine: Arc<KvEngine>) -> anyhow::Result<Self> {
        let mut groups = HashMap::new();
        for shard in cluster.owned_shards() {
            let group = RaftKv::single_node(cluster.node_id as NodeId, engine.clone()).await?;
            groups.insert(shard, group);
        }
        Ok(Self {
            cluster,
            groups,
            engine,
        })
    }

    /// Route `op` to the group owning `key`'s shard, proposing it through Raft.
    pub async fn write(&self, key: &str, op: WalOp) -> anyhow::Result<Response> {
        let shard = self.cluster.shard_for(key);
        let group = self.groups.get(&shard).ok_or_else(|| {
            anyhow::anyhow!("shard {shard} for key '{key}' not owned by this node")
        })?;
        group.write(op).await
    }

    /// Whether this node owns `key`'s shard.
    pub fn owns(&self, key: &str) -> bool {
        self.cluster.owns(key)
    }

    /// Number of per-shard groups this node runs.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}
