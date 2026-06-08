//! Per-shard Raft skeleton.
//!
//! v1 ships the **surface** without the consensus machinery —
//! readiness, the peer DNS map, role inspection, and the wire shape of
//! the cluster state are all in place so callers and operators can
//! depend on them. The actual log/election/snapshot is the next
//! storage-tier slice (openraft).
//!
//! Until that lands, the single-pod build behaves as if the pod is the
//! permanent leader of its shard (writes go through, reads return the
//! freshest data). Multi-pod K8s deployments will fail to make progress
//! on the data plane — by design — until the openraft wiring lands.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::ClusterConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum RaftRole {
    Leader,
    Follower,
    Learner,
    Candidate,
}

/// Read-consistency requirement set by a client request via the
/// `X-Read-Consistency` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReadConsistency {
    /// Default — only the shard leader may answer.
    Leader,
    /// A follower may answer if its replication lag is below the bound
    /// (ms). Carried in `bounded(ms)` form on the wire.
    Bounded(u64),
    /// Any replica is allowed (potentially stale).
    Any,
}

impl ReadConsistency {
    pub fn from_header(raw: Option<&str>) -> Self {
        let Some(v) = raw else {
            return Self::Leader;
        };
        let v = v.trim().to_ascii_lowercase();
        if v == "leader" {
            Self::Leader
        } else if v == "any" {
            Self::Any
        } else if let Some(ms) = v
            .strip_prefix("bounded(")
            .and_then(|t| t.strip_suffix(')'))
            .and_then(|n| n.parse::<u64>().ok())
        {
            Self::Bounded(ms)
        } else {
            // Unknown values fall back to the safest setting.
            Self::Leader
        }
    }
}

/// Peer list for one shard. The address scheme is the same for every
/// deployment: `lumen-{ordinal}.{headless_service}:{port}` where the
/// pod ordinal is `replica * shard_count + shard`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftGroup {
    pub shard_index: u32,
    pub peers: Vec<PeerAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PeerAddr {
    pub pod_name: String,
    pub host: String,
    pub raft_port: u16,
    pub client_port: u16,
    pub role: RaftRole,
}

impl RaftGroup {
    pub fn from_config(
        cfg: &ClusterConfig,
        prefix: &str,
        headless_service: &str,
        raft_port: u16,
        client_port: u16,
    ) -> anyhow::Result<Self> {
        let shard = cfg.shard_index()?;
        let mut peers = Vec::with_capacity(cfg.replicas_per_shard as usize);
        for replica in 0..cfg.replicas_per_shard {
            let ordinal = replica * cfg.shard_count + shard;
            let pod_name = format!("{prefix}-{ordinal}");
            let host = format!("{pod_name}.{headless_service}");
            let role = if replica < cfg.voter_count {
                if replica == 0 {
                    // Stub: pod 0 always claims leader. Real Raft
                    // will own this assignment.
                    RaftRole::Leader
                } else {
                    RaftRole::Follower
                }
            } else {
                RaftRole::Learner
            };
            peers.push(PeerAddr {
                pod_name,
                host,
                raft_port,
                client_port,
                role,
            });
        }

        // Local-dev override: `LUMEN_PEERS=host:peer-port,host:peer-port,...`
        // replaces the K8s headless-DNS addresses with explicit
        // host:port pairs. Useful for running a 3-pod cluster on a
        // single machine; index N maps to replica N in this shard.
        if let Ok(raw) = std::env::var("LUMEN_PEERS") {
            let overrides: Vec<&str> = raw
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            for (i, peer) in peers.iter_mut().enumerate() {
                if let Some(addr) = overrides.get(i) {
                    if let Some((host, port)) = addr.rsplit_once(':') {
                        peer.host = host.to_string();
                        peer.raft_port = port.parse().unwrap_or(peer.raft_port);
                    } else {
                        peer.host = (*addr).to_string();
                    }
                }
            }
        }

        Ok(Self {
            shard_index: shard,
            peers,
        })
    }

    pub fn leader(&self) -> Option<&PeerAddr> {
        self.peers.iter().find(|p| p.role == RaftRole::Leader)
    }
}

/// Live cluster snapshot for `/debug/cluster`. Cheap to clone; updated
/// in place from background tasks once the openraft wiring lands.
#[derive(Debug)]
pub struct ClusterState {
    pub pod_name: String,
    pub shard_index: u32,
    pub replica_index: u32,
    pub role: RaftRole,
    pub group: RaftGroup,
    pub applied_index: AtomicU64,
    pub leader_term: AtomicU64,
    pub replication_lag_ms: AtomicU64,
}

impl ClusterState {
    pub fn new(cfg: &ClusterConfig, group: RaftGroup) -> anyhow::Result<Self> {
        let role = if cfg.is_voter()? {
            if cfg.replica_index()? == 0 {
                RaftRole::Leader
            } else {
                RaftRole::Follower
            }
        } else {
            RaftRole::Learner
        };
        Ok(Self {
            pod_name: cfg.pod_name.clone(),
            shard_index: cfg.shard_index()?,
            replica_index: cfg.replica_index()?,
            role,
            group,
            applied_index: AtomicU64::new(0),
            leader_term: AtomicU64::new(1),
            replication_lag_ms: AtomicU64::new(0),
        })
    }

    pub fn snapshot(&self) -> ClusterStateView {
        ClusterStateView {
            pod_name: self.pod_name.clone(),
            shard_index: self.shard_index,
            replica_index: self.replica_index,
            role: self.role,
            peers: self.group.peers.clone(),
            applied_index: self.applied_index.load(Ordering::Relaxed),
            leader_term: self.leader_term.load(Ordering::Relaxed),
            replication_lag_ms: self.replication_lag_ms.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClusterStateView {
    pub pod_name: String,
    pub shard_index: u32,
    pub replica_index: u32,
    pub role: RaftRole,
    pub peers: Vec<PeerAddr>,
    pub applied_index: u64,
    pub leader_term: u64,
    pub replication_lag_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_consistency_from_header() {
        assert_eq!(
            ReadConsistency::from_header(Some("leader")),
            ReadConsistency::Leader
        );
        assert_eq!(
            ReadConsistency::from_header(Some("any")),
            ReadConsistency::Any
        );
        assert_eq!(
            ReadConsistency::from_header(Some("Bounded(250)")),
            ReadConsistency::Bounded(250)
        );
        assert_eq!(
            ReadConsistency::from_header(Some("gibberish")),
            ReadConsistency::Leader
        );
        assert_eq!(ReadConsistency::from_header(None), ReadConsistency::Leader);
    }

    use crate::config::ClusterConfig;
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn cfg(shards: u32, replicas: u32, voters: u32, pod: &str) -> ClusterConfig {
        ClusterConfig {
            shard_count: shards,
            replicas_per_shard: replicas,
            voter_count: voters,
            pod_name: pod.into(),
        }
    }

    fn clear_lumen_peers() {
        unsafe {
            std::env::remove_var("LUMEN_PEERS");
        }
    }

    #[test]
    fn raft_group_from_config_enumerates_peers_in_shard() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_lumen_peers();
        // 3 shards × 3 replicas, this pod is lumen-4 → shard 1, replica 1.
        // The group's peers are the 3 replicas of shard 1: ordinals 1, 4, 7.
        let g = RaftGroup::from_config(&cfg(3, 3, 3, "lumen-4"), "lumen", "lumen-peer", 8082, 8080)
            .unwrap();
        assert_eq!(g.shard_index, 1);
        assert_eq!(g.peers.len(), 3);
        assert_eq!(g.peers[0].pod_name, "lumen-1");
        assert_eq!(g.peers[1].pod_name, "lumen-4");
        assert_eq!(g.peers[2].pod_name, "lumen-7");
        // Hostnames go through the headless service suffix.
        for p in &g.peers {
            assert!(p.host.ends_with(".lumen-peer"), "host={}", p.host);
            assert_eq!(p.raft_port, 8082);
            assert_eq!(p.client_port, 8080);
        }
    }

    #[test]
    fn raft_group_marks_first_voter_as_leader_and_rest_as_followers() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_lumen_peers();
        let g = RaftGroup::from_config(&cfg(1, 5, 3, "lumen-0"), "lumen", "lumen-peer", 8082, 8080)
            .unwrap();
        assert_eq!(g.peers[0].role, RaftRole::Leader);
        assert_eq!(g.peers[1].role, RaftRole::Follower);
        assert_eq!(g.peers[2].role, RaftRole::Follower);
        // 4th and 5th replicas exceed voter_count=3 → learners.
        assert_eq!(g.peers[3].role, RaftRole::Learner);
        assert_eq!(g.peers[4].role, RaftRole::Learner);
        assert_eq!(g.leader().unwrap().pod_name, "lumen-0");
    }

    #[test]
    fn raft_group_lumen_peers_override_replaces_dns() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var(
                "LUMEN_PEERS",
                "127.0.0.1:9080,127.0.0.1:9081,127.0.0.1:9082",
            );
        }
        let g = RaftGroup::from_config(&cfg(1, 3, 3, "lumen-0"), "lumen", "lumen-peer", 8082, 8080)
            .unwrap();
        assert_eq!(g.peers[0].host, "127.0.0.1");
        assert_eq!(g.peers[0].raft_port, 9080);
        assert_eq!(g.peers[1].raft_port, 9081);
        assert_eq!(g.peers[2].raft_port, 9082);
        clear_lumen_peers();
    }

    #[test]
    fn raft_group_lumen_peers_partial_override_keeps_remaining_dns() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("LUMEN_PEERS", "127.0.0.1:9080");
        }
        let g = RaftGroup::from_config(&cfg(1, 3, 3, "lumen-0"), "lumen", "lumen-peer", 8082, 8080)
            .unwrap();
        assert_eq!(g.peers[0].host, "127.0.0.1");
        // Peer 1 and 2 keep the headless DNS form.
        assert!(g.peers[1].host.ends_with(".lumen-peer"));
        assert!(g.peers[2].host.ends_with(".lumen-peer"));
        clear_lumen_peers();
    }

    #[test]
    fn leader_returns_none_when_no_voter_in_group() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_lumen_peers();
        // 0 voters → every replica is a learner; leader() returns None.
        let g = RaftGroup::from_config(&cfg(1, 3, 0, "lumen-0"), "lumen", "lumen-peer", 8082, 8080)
            .unwrap();
        assert!(g.leader().is_none());
    }

    #[test]
    fn cluster_state_view_round_trips() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_lumen_peers();
        let c = cfg(1, 3, 3, "lumen-1");
        let group = RaftGroup::from_config(&c, "lumen", "lumen-peer", 8082, 8080).unwrap();
        let st = ClusterState::new(&c, group).unwrap();
        let v = st.snapshot();
        assert_eq!(v.pod_name, "lumen-1");
        assert_eq!(v.replica_index, 1);
        assert_eq!(v.role, RaftRole::Follower);
        // applied_index / term default to 0 / 1.
        assert_eq!(v.applied_index, 0);
        assert_eq!(v.leader_term, 1);
    }
}
