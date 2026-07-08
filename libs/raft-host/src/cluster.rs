// SPEC-MANAGED: libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
pub fn replica_mode() -> bool {
    std::env::var("REPLICAS_PER_SHARD")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(1)
        > 1
}

/// The scalar shard/replica/voter derivation from the standard downward-API
/// quartet (`SHARD_COUNT`, `REPLICAS_PER_SHARD`, `VOTER_COUNT`, `POD_NAME`) —
/// the piece [`ClusterTopology::from_env`] shares with a caller that only
/// needs the scalars, not peer URLs (e.g. lumen's `ClusterConfig`, which
/// stays compiled outside the `raft-wal` feature; #1002).
#[derive(Debug, Clone)]
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
pub struct ClusterDims {
    pub shard_count: u32,
    pub replicas_per_shard: u32,
    pub voter_count: u32,
    pub pod_name: String,
}

/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
impl ClusterDims {
    /// Read the standard downward-API quartet.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            shard_count: parse_env("SHARD_COUNT")?,
            replicas_per_shard: parse_env("REPLICAS_PER_SHARD")?,
            voter_count: parse_env("VOTER_COUNT")?,
            pod_name: std::env::var("POD_NAME").context("POD_NAME not set")?,
        })
    }

    /// The trailing `-<N>` ordinal in `pod_name` — the StatefulSet identity.
    pub fn pod_ordinal(&self) -> Result<u32> {
        let (_, suffix) = self
            .pod_name
            .rsplit_once('-')
            .context("POD_NAME has no '-<ordinal>' suffix")?;
        suffix
            .parse()
            .with_context(|| format!("POD_NAME ordinal '{suffix}' is not a u32"))
    }

    /// `ordinal % shard_count` — which shard this pod belongs to.
    pub fn shard_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? % self.shard_count)
    }

    /// `ordinal / shard_count` — this pod's replica index within its shard
    /// (== the raft node id).
    pub fn replica_index(&self) -> Result<u32> {
        Ok(self.pod_ordinal()? / self.shard_count)
    }

    /// Whether this replica votes (`replica_index < voter_count`); the rest
    /// are learners.
    pub fn is_voter(&self) -> Result<bool> {
        Ok(self.replica_index()? < self.voter_count)
    }
}

/// The StatefulSet pod ordinal for `replica` within a shard
/// (`replica * shard_count + shard_index`) — the peer-DNS math shared by
/// every peer enumeration: [`ClusterTopology::from_env`]'s peer URLs and a
/// caller's own richer per-peer record (e.g. lumen's `RaftGroup`/`PeerAddr`).
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
pub fn peer_ordinal(shard_count: u32, shard_index: u32, replica: u32) -> u32 {
    replica * shard_count + shard_index
}

/// Parse a `LUMEN_PEERS`-style override env var (`host[:port],host[:port],...`,
/// empty entries filtered) into an `index -> host[:port]` override list.
/// Empty when `env_var` is unset — callers then use the DNS-derived
/// addresses unmodified. Shared by [`ClusterTopology::from_env`] and any
/// caller enumerating its own peer records with the same override contract.
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
pub fn parse_peer_overrides(env_var: &str) -> Vec<String> {
    std::env::var(env_var)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// One raft group's topology, derived from the StatefulSet downward API.
#[derive(Debug, Clone)]
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
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

/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-cluster-rs.md#source
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
        let dims = ClusterDims::from_env()?;
        let shard_count = dims.shard_count;
        let replicas_per_shard = dims.replicas_per_shard;
        let voter_count = dims.voter_count;
        let shard_index = dims.shard_index()?;
        let node_id = dims.replica_index()? as NodeId;

        // pod ordinal → (shard, replica) is pure integer math, so peers are found
        // via headless DNS with no discovery service. `index N → replica N`.
        let overrides = parse_peer_overrides(peers_override);

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
                    let ordinal = peer_ordinal(shard_count, shard_index, replica);
                    format!("http://{prefix}-{ordinal}.{headless_service}:{peer_port}")
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

    fn dims(shard_count: u32, replicas_per_shard: u32, voter_count: u32, pod: &str) -> ClusterDims {
        ClusterDims {
            shard_count,
            replicas_per_shard,
            voter_count,
            pod_name: pod.into(),
        }
    }

    #[test]
    fn cluster_dims_derives_shard_and_replica_from_pod_ordinal() {
        // 3 shards × 3 replicas: pod-7 → shard 1, replica 2.
        let d = dims(3, 3, 3, "svc-7");
        assert_eq!(d.pod_ordinal().unwrap(), 7);
        assert_eq!(d.shard_index().unwrap(), 1);
        assert_eq!(d.replica_index().unwrap(), 2);
        assert!(d.is_voter().unwrap());

        let d = dims(3, 3, 2, "svc-8");
        assert_eq!(d.shard_index().unwrap(), 2);
        assert_eq!(d.replica_index().unwrap(), 2);
        assert!(
            !d.is_voter().unwrap(),
            "replica 2 is a learner when voter_count=2"
        );
    }

    #[test]
    fn cluster_dims_pod_ordinal_rejects_bad_suffix() {
        assert!(dims(3, 3, 3, "svc-").pod_ordinal().is_err());
        assert!(dims(3, 3, 3, "svc-abc").pod_ordinal().is_err());
        assert!(dims(3, 3, 3, "svc").pod_ordinal().is_err());
    }

    #[test]
    fn peer_ordinal_matches_replica_times_shard_count_plus_shard() {
        assert_eq!(peer_ordinal(3, 1, 2), 7); // 2*3+1
        assert_eq!(peer_ordinal(1, 0, 4), 4);
    }

    #[test]
    fn parse_peer_overrides_splits_trims_and_filters_empty() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("TEST_PEERS");
        assert!(parse_peer_overrides("TEST_PEERS").is_empty());

        std::env::set_var("TEST_PEERS", " a:1, b:2 ,,c:3");
        assert_eq!(
            parse_peer_overrides("TEST_PEERS"),
            vec!["a:1".to_string(), "b:2".to_string(), "c:3".to_string()]
        );
        std::env::remove_var("TEST_PEERS");
    }
}
// CODEGEN-END
