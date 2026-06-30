//! raft-host-backed consensus (HA phase C). keep's engine is wired as a
//! [`raft_host::RaftStateMachine`] so writes go through the shared driver
//! (propose → commit → apply); adopting [`RaftHost`] gives keep the proven h2c
//! peer transport + sole-applier read-your-write it previously lacked.
//!
//! - [`KvStateMachine`] is the engine fronting **one shard's** raft group: the
//!   command is a [`WalOp`] (the same type the WAL + recovery already use) and
//!   apply reuses the WAL-replay path ([`RecoveryManager::apply_one`]).
//!   `snapshot`/`restore` ride `dump_values`/`load_values`, filtered to the
//!   shard's keyspace so each per-shard snapshot carries only its own keys.
//! - [`ShardHosts`] runs **one [`RaftHost`] per owned shard** — keep's
//!   distinguishing complexity vs the single-host services (lumen/relay). A
//!   write routes to its key's shard host; each host's peer router mounts under
//!   `/shard/{id}` so the Vote/Append/InstallSnapshot RPCs ride the serve port.
//!
//! This replaces the earlier hand-rolled `RaftKv`/`ShardedRaft` glue (a
//! single-node-only `raft_core` group with no transport); the routing shell
//! ([`ClusterConfig`]) is unchanged. See HA.md.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use raft_host::{
    FsyncPolicy, HostConfig, Index, Membership, NodeId, RaftHost, RaftStateMachine, RaftStore,
    SnapshotPolicy,
};
use serde::{Deserialize, Serialize};

use crate::cluster::ClusterConfig;
use crate::engine::KvEngine;
use crate::persistence::format::WalOp;
use crate::persistence::recovery::RecoveryManager;
use crate::types::KvValue;

/// How many committed entries a shard's host applies before it captures a
/// snapshot and compacts the raft log (bounds the log; arms InstallSnapshot for
/// a lagging/fresh replica).
const SNAPSHOT_EVERY: u64 = 1024;

/// A shard's keyspace snapshot: the live values whose key hashes to this shard,
/// tagged with the raft index they are current as of (so `restore` can set the
/// applied index, per the [`RaftStateMachine`] contract).
#[derive(Serialize, Deserialize)]
struct ShardSnapshot {
    up_to: Index,
    values: Vec<(String, KvValue)>,
}

/// keep's [`KvEngine`] driven as a [`RaftStateMachine`] for one shard's group.
///
/// Every shard on a node shares the **one** engine; the state machine is scoped
/// to a `shard` so its snapshot/restore touch only that shard's keys (apply only
/// ever receives that shard's ops, because writes are routed by key). `applied`
/// is this shard group's own raft head — each group has an independent log.
pub struct KvStateMachine {
    engine: Arc<KvEngine>,
    cluster: ClusterConfig,
    shard: u32,
    applied: AtomicU64,
}

impl KvStateMachine {
    /// Wrap `engine` as the state machine for `shard` of `cluster`.
    pub fn new(engine: Arc<KvEngine>, cluster: ClusterConfig, shard: u32) -> Arc<Self> {
        Arc::new(Self {
            engine,
            cluster,
            shard,
            applied: AtomicU64::new(0),
        })
    }
}

impl RaftStateMachine for KvStateMachine {
    fn apply(&self, index: Index, command: &[u8]) -> Result<()> {
        // Decode the committed command and replay it through the exact WAL
        // recovery path. A bad decode / engine error no-ops the entry (logged)
        // but still advances `applied` so the log keeps moving (sole applier).
        match serde_json::from_slice::<WalOp>(command) {
            Ok(op) => {
                if let Err(e) = RecoveryManager::apply_one(&self.engine, &op) {
                    tracing::warn!(index, error = %e, "raft: apply_one error (entry no-ops)");
                }
            }
            Err(e) => {
                tracing::warn!(index, error = %e, "raft: undecodable command (entry no-ops)")
            }
        }
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        let values = self
            .engine
            .dump_values()
            .into_iter()
            .filter(|(k, _)| self.cluster.shard_for(k) == self.shard)
            .collect();
        Ok(serde_json::to_vec(&ShardSnapshot {
            up_to: self.applied_index(),
            values,
        })?)
    }

    fn restore(&self, snapshot: &[u8]) -> Result<()> {
        let snap: ShardSnapshot = serde_json::from_slice(snapshot)?;
        self.engine.load_values(snap.values);
        self.applied.store(snap.up_to, Ordering::Release);
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

/// One [`RaftHost`] per owned shard; a write routes to its key's shard host.
pub struct ShardHosts {
    cluster: ClusterConfig,
    hosts: HashMap<u32, Arc<RaftHost>>,
    pub engine: Arc<KvEngine>,
}

impl ShardHosts {
    /// Spawn one host per `cluster.owned_shards()` shard, persisting each group's
    /// hard state under `{data_dir}/raft/shard-{id}`. `replicas_per_shard <= 1`
    /// runs each shard as a sole-voter single-node group (read-your-write, no
    /// peers); `> 1` derives the shard's replica membership + `/shard/{id}` peer
    /// URLs for the shared h2c transport.
    pub async fn new(
        cluster: ClusterConfig,
        engine: Arc<KvEngine>,
        data_dir: &Path,
        replicas_per_shard: u32,
    ) -> Result<Self> {
        Self::with_snapshot_every(
            cluster,
            engine,
            data_dir,
            replicas_per_shard,
            SNAPSHOT_EVERY,
        )
        .await
    }

    /// As [`new`](Self::new), with the snapshot/compaction cadence overridden
    /// (the snapshot-catch-up test drives a small threshold so InstallSnapshot
    /// fires quickly).
    pub async fn with_snapshot_every(
        cluster: ClusterConfig,
        engine: Arc<KvEngine>,
        data_dir: &Path,
        replicas_per_shard: u32,
        snapshot_every: u64,
    ) -> Result<Self> {
        let mut hosts = HashMap::new();
        for shard in cluster.owned_shards() {
            let (node_id, membership, peers) = shard_topology(&cluster, shard, replicas_per_shard);
            let dir = data_dir.join("raft").join(format!("shard-{shard}"));
            std::fs::create_dir_all(&dir)?;
            let store = RaftStore::open(
                dir.to_str()
                    .ok_or_else(|| anyhow::anyhow!("raft data dir is not valid UTF-8"))?,
                node_id,
                FsyncPolicy::Always,
            )?;
            let sm = KvStateMachine::new(engine.clone(), cluster.clone(), shard);
            let host = RaftHost::spawn(
                node_id,
                membership,
                peers,
                store,
                sm as Arc<dyn RaftStateMachine>,
                HostConfig {
                    snapshot: SnapshotPolicy::EveryEntries(snapshot_every),
                    ..Default::default()
                },
            );
            hosts.insert(shard, Arc::new(host));
        }
        Ok(Self {
            cluster,
            hosts,
            engine,
        })
    }

    /// Route `op` to the host owning `key`'s shard and propose it through Raft;
    /// resolves with the assigned raft index once **this node's** engine has
    /// applied it (read-your-write). Errors if `key`'s shard is not owned here.
    pub async fn write(&self, key: &str, op: WalOp) -> Result<Index> {
        let host = self
            .host_for(key)
            .ok_or_else(|| anyhow::anyhow!("shard for key '{key}' not owned by this node"))?;
        host.propose(serde_json::to_vec(&op)?).await
    }

    /// Merge every shard host's peer router, each nested under `/shard/{id}`, so
    /// the Vote/Append/InstallSnapshot + leader-redirect RPCs ride the service's
    /// h2c serve port. Peer base URLs carry the matching `/shard/{id}` prefix.
    pub fn router(&self) -> Router {
        let mut app = Router::new();
        for (shard, host) in &self.hosts {
            app = app.merge(Router::new().nest(&format!("/shard/{shard}"), host.router()));
        }
        app
    }

    /// Whether this node owns `key`'s shard.
    pub fn owns(&self, key: &str) -> bool {
        self.cluster.owns(key)
    }

    /// Number of per-shard hosts this node runs.
    pub fn host_count(&self) -> usize {
        self.hosts.len()
    }

    /// The host driving `key`'s shard group (for status / direct propose).
    pub fn host_for(&self, key: &str) -> Option<&Arc<RaftHost>> {
        self.hosts.get(&self.cluster.shard_for(key))
    }
}

/// Derive a shard group's raft node id, membership, and peer URLs.
///
/// Single-node (`replicas_per_shard <= 1`): a sole voter (`node 0`, no peers) —
/// the group commits locally (read-your-write with no transport). HA (`> 1`):
/// shard `S`'s replicas are the nodes `(owner + r) % node_count` for
/// `r in 0..replicas`; this node's raft id is its replica index `r`, every
/// replica is a voter, and each peer URL is the peer node's base address with a
/// `/shard/{S}` suffix so it matches the nested peer router.
fn shard_topology(
    cluster: &ClusterConfig,
    shard: u32,
    replicas_per_shard: u32,
) -> (NodeId, Membership, HashMap<NodeId, String>) {
    if replicas_per_shard <= 1 {
        return (
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
        );
    }
    let node_count = cluster.node_count.max(1) as u32;
    let replicas = replicas_per_shard.min(node_count);
    let owner = cluster.owner_of_shard(shard) as u32;
    let mut node_id: NodeId = 0;
    let mut peers = HashMap::new();
    for r in 0..replicas {
        let node = ((owner + r) % node_count) as usize;
        let rid = r as NodeId;
        if node == cluster.node_id {
            node_id = rid;
        } else if let Some(base) = cluster.peers.get(node) {
            peers.insert(rid, format!("{}/shard/{shard}", base.trim_end_matches('/')));
        }
    }
    let membership = Membership {
        voters: (0..replicas as NodeId).collect(),
        learners: vec![],
    };
    (node_id, membership, peers)
}
