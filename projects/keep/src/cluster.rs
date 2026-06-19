//! Cluster topology & sharding (HA phase A: horizontal scale-out).
//!
//! keep runs as a StatefulSet of independent stores (`keep-0..keep-N`). The
//! keyspace is split into `shard_count` virtual shards mapped onto `node_count`
//! nodes; a client routes a key with `crc32(key) % shard_count -> shard ->
//! node`. This module is the addressing/membership substrate that both the
//! client-routing story and a future raft consensus layer build on. It does NOT
//! itself replicate or coordinate — that's phase C (see HA.md).
//!
//! Single-node default (node_count = 1): one node owns every shard.

use std::sync::Arc;

use serde::Serialize;
use utoipa::ToSchema;

/// Cluster membership + sharding parameters. Install-time constants (changing
/// `shard_count` on a live cluster reshuffles ownership — treat as fixed).
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// This node's ordinal (k8s StatefulSet pod index `keep-<id>`).
    pub node_id: usize,
    /// Total nodes in the cluster.
    pub node_count: usize,
    /// Virtual shard count (>= node_count; the client-side routing fan-out).
    pub shard_count: u32,
    /// Peer base URLs, index = node ordinal (for a future replication layer).
    pub peers: Vec<String>,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            node_id: 0,
            node_count: 1,
            shard_count: 1,
            peers: Vec::new(),
        }
    }
}

impl ClusterConfig {
    /// Build from a node id / count / shard count, deriving k8s peer DNS names
    /// (`keep-<i>.<svc>`) when no explicit peers are given.
    pub fn new(node_id: usize, node_count: usize, shard_count: u32, peers: Vec<String>) -> Self {
        let node_count = node_count.max(1);
        let shard_count = shard_count.max(node_count as u32).max(1);
        Self {
            node_id: node_id.min(node_count - 1),
            node_count,
            shard_count,
            peers,
        }
    }

    /// Shard a key falls in: `crc32(key) % shard_count`.
    pub fn shard_for(&self, key: &str) -> u32 {
        crc32fast::hash(key.as_bytes()) % self.shard_count
    }

    /// Node that owns a shard: `shard % node_count`.
    pub fn owner_of_shard(&self, shard: u32) -> usize {
        (shard % self.node_count as u32) as usize
    }

    /// Whether this node owns `key`.
    pub fn owns(&self, key: &str) -> bool {
        self.owner_of_shard(self.shard_for(key)) == self.node_id
    }

    /// Shard ids this node owns.
    pub fn owned_shards(&self) -> Vec<u32> {
        (0..self.shard_count)
            .filter(|s| self.owner_of_shard(*s) == self.node_id)
            .collect()
    }

    pub fn snapshot(&self) -> ClusterState {
        ClusterState {
            node_id: self.node_id,
            node_count: self.node_count,
            shard_count: self.shard_count,
            owned_shards: self.owned_shards().len(),
            peers: self.peers.clone(),
            // Phase A has no consensus: every node is its own (single-shard-group) leader.
            mode: if self.node_count > 1 { "sharded" } else { "single" }.to_string(),
        }
    }
}

/// Public cluster-state view (served at `/cluster`).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClusterState {
    pub node_id: usize,
    pub node_count: usize,
    pub shard_count: u32,
    pub owned_shards: usize,
    pub peers: Vec<String>,
    /// `single` or `sharded` (phase A). `raft` arrives with phase C.
    pub mode: String,
}

/// Shared cluster handle in [`crate::http::AppState`].
pub type Cluster = Arc<ClusterConfig>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_node_owns_everything() {
        let c = ClusterConfig::default();
        assert!(c.owns("anything"));
        assert_eq!(c.owned_shards(), vec![0]);
    }

    #[test]
    fn sharding_partitions_keys_across_nodes() {
        // 3 nodes, 12 shards. Each node owns a disjoint subset; union = all keys.
        let nodes: Vec<ClusterConfig> =
            (0..3).map(|i| ClusterConfig::new(i, 3, 12, vec![])).collect();
        for k in ["a", "b", "user:1", "result:job-42", "x"] {
            let owners: Vec<bool> = nodes.iter().map(|n| n.owns(k)).collect();
            assert_eq!(owners.iter().filter(|&&o| o).count(), 1, "key {k} must have exactly one owner");
        }
        // Ownership is stable + disjoint.
        let total: usize = nodes.iter().map(|n| n.owned_shards().len()).sum();
        assert_eq!(total, 12);
    }

    #[test]
    fn shard_count_floored_to_node_count() {
        let c = ClusterConfig::new(0, 5, 2, vec![]);
        assert!(c.shard_count >= 5);
    }
}
