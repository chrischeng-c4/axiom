//! k8s-native cluster topology + auto-mode for the raft host.
//!
//! Every raft_core service derives the same thing from the StatefulSet downward
//! API: which mode to run (single-node vs replica/HA), this node's id, the
//! group membership, and the peer URLs. This module centralizes it so services
//! compose it instead of hand-rolling the ordinal math + peer-DNS each time.

use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::{Membership, NodeId};

/// Whether the StatefulSet runs in replica/HA mode: `true` when
/// `REPLICAS_PER_SHARD > 1`. A single replica — or no cluster context (the env
/// unset, e.g. local dev) — is single-node. This is the **auto-mode** switch: a
/// service defaults to single-node and turns on raft only when k8s scales it out.
pub fn replica_mode() -> bool {
    std::env::var("REPLICAS_PER_SHARD")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        > 1
}

/// One raft group's topology, derived from the StatefulSet downward API.
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    /// This node's id within its shard's group = the replica index.
    pub node_id: NodeId,
    /// Voters `0..voter_count`, learners the rest.
    pub membership: Membership,
    /// Peer base URLs (`NodeId → http://host:port`), excluding self.
    pub peers: HashMap<NodeId, String>,
    pub replicas_per_shard: u32,
    pub shard_index: u32,
}

impl ClusterTopology {
    /// Build from the standard downward-API env (`POD_NAME`, `SHARD_COUNT`,
    /// `REPLICAS_PER_SHARD`, `VOTER_COUNT`) and a peer-DNS template
    /// (`<prefix>-<ordinal>.<headless_service>:<peer_port>`). `peers_override` is
    /// the name of an env var (e.g. `LUMEN_PEERS`) holding `host[:port],...` that
    /// replaces the DNS addresses — for running a multi-node group on one machine.
    pub fn from_env(
        prefix: &str,
        headless_service: &str,
        peer_port: u16,
        peers_override: &str,
    ) -> Result<Self> {
        let shard_count: u32 = parse_env("SHARD_COUNT")?;
        let replicas_per_shard: u32 = parse_env("REPLICAS_PER_SHARD")?;
        let voter_count: u32 = parse_env("VOTER_COUNT")?;
        let pod_name = std::env::var("POD_NAME").context("POD_NAME not set")?;
        let ordinal: u32 = pod_name
            .rsplit_once('-')
            .context("POD_NAME has no '-<ordinal>' suffix")?
            .1
            .parse()
            .context("POD_NAME ordinal is not a u32")?;
        let shard_index = ordinal % shard_count;
        let node_id = (ordinal / shard_count) as NodeId; // replica index

        // pod ordinal → (shard, replica) is pure integer math, so peers are found
        // via headless DNS with no discovery service. `index N → replica N`.
        let overrides: Vec<String> = std::env::var(peers_override)
            .ok()
            .map(|raw| {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let mut peers = HashMap::new();
        for replica in 0..replicas_per_shard {
            let id = replica as NodeId;
            if id == node_id {
                continue;
            }
            let url = match overrides.get(replica as usize) {
                Some(addr) if addr.contains(':') => format!("http://{addr}"),
                Some(addr) => format!("http://{addr}:{peer_port}"),
                None => {
                    let peer_ordinal = replica * shard_count + shard_index;
                    format!("http://{prefix}-{peer_ordinal}.{headless_service}:{peer_port}")
                }
            };
            peers.insert(id, url);
        }

        let membership = Membership {
            voters: (0..voter_count as NodeId).collect(),
            learners: (voter_count as NodeId..replicas_per_shard as NodeId).collect(),
        };
        Ok(Self {
            node_id,
            membership,
            peers,
            replicas_per_shard,
            shard_index,
        })
    }
}

fn parse_env(key: &str) -> Result<u32> {
    std::env::var(key)
        .with_context(|| format!("{key} not set"))?
        .parse()
        .with_context(|| format!("{key} must be a u32"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // The standard env vars are process-global; serialize the env tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn replica_mode_defaults_to_single_node() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("REPLICAS_PER_SHARD");
        assert!(!replica_mode());
        std::env::set_var("REPLICAS_PER_SHARD", "1");
        assert!(!replica_mode());
        std::env::set_var("REPLICAS_PER_SHARD", "3");
        assert!(replica_mode());
        std::env::remove_var("REPLICAS_PER_SHARD");
    }

    #[test]
    fn topology_from_env_with_local_override() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("SHARD_COUNT", "1");
        std::env::set_var("REPLICAS_PER_SHARD", "3");
        std::env::set_var("VOTER_COUNT", "3");
        std::env::set_var("POD_NAME", "svc-1");
        std::env::set_var("SVC_PEERS", "10.0.0.0:9001,10.0.0.1:9002,10.0.0.2:9003");
        let t = ClusterTopology::from_env("svc", "svc-headless", 7000, "SVC_PEERS").unwrap();
        assert_eq!(t.node_id, 1);
        assert_eq!(t.membership.voters, vec![0, 1, 2]);
        // self (id 1) excluded; peers point at the override addresses.
        assert_eq!(t.peers.get(&0).unwrap(), "http://10.0.0.0:9001");
        assert_eq!(t.peers.get(&2).unwrap(), "http://10.0.0.2:9003");
        assert!(t.peers.get(&1).is_none());
        for k in [
            "SHARD_COUNT",
            "REPLICAS_PER_SHARD",
            "VOTER_COUNT",
            "POD_NAME",
            "SVC_PEERS",
        ] {
            std::env::remove_var(k);
        }
    }
}
