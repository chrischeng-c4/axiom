---
id: projects-lumen-src-raft-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/raft.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/raft.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReadConsistency` | raft_host (re-export) | enum | pub use | 36 |  |
| `ClusterState` | projects/lumen/src/raft.rs | struct | pub | 162 |  |
| `ClusterStateView` | projects/lumen/src/raft.rs | struct | pub | 213 |  |
| `PeerAddr` | projects/lumen/src/raft.rs | struct | pub | 72 |  |
| `RaftGroup` | projects/lumen/src/raft.rs | struct | pub | 65 |  |
| `RaftRole` | projects/lumen/src/raft.rs | enum | pub | 41 |  |
| `from_config` | projects/lumen/src/raft.rs | function | pub | 95 | from_config(         cfg: &ClusterConfig,         prefix: &str,         headless_service: &str,         raft_port: u16,         client_port: u16,     ) -> anyhow::Result<Self> |
| `leader` | projects/lumen/src/raft.rs | function | pub | 153 | leader(&self) -> Option<&PeerAddr> |
| `new` | projects/lumen/src/raft.rs | function | pub | 175 | new(cfg: &ClusterConfig, group: RaftGroup) -> anyhow::Result<Self> |
| `snapshot` | projects/lumen/src/raft.rs | function | pub | 197 | snapshot(&self) -> ClusterStateView |

`ReadConsistency` (header name + `from_header` parsing) and the
`RaftRole`/`PeerAddr`/`ClusterStateView` shapes are now canonically defined
in `libs/raft-host` (`read_consistency.rs` / `view.rs`, #1003); this file
re-exports `ReadConsistency` directly and keeps `utoipa::ToSchema`-deriving
wrappers with `From<raft_host::*>` conversions for the other three so
`/openapi.json` stays byte-identical.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Per-shard replication surface.
//!
//! This module currently carries the public cluster-state DTOs — readiness,
//! peer DNS map, role inspection, read-consistency parsing, and the wire shape
//! of `/debug/cluster`. The next implementation slice wires this surface to
//! `libs/raft-core` so multi-pod Lumen owns write ordering and primary/replica
//! synchronization itself.
//!
//! Lumen's multi-pod auto path uses Lumen-owned primary/replica replication.
//!
//! `RaftGroup::from_config`'s peer enumeration (pod-ordinal math + the
//! `LUMEN_PEERS` override parsing) delegates to `libs/raft-host::cluster`
//! (#1002) so it can't drift from `raft_host::cluster::ClusterTopology::
//! from_env`, the implementation the actual raft-wal peer wiring uses.
//!
//! The read-consistency header contract and the role/cluster-view model are
//! the same shape every raft_core service exposes, so their canonical
//! definitions now live in `libs/raft-host` (#1003). `ReadConsistency` is
//! re-exported directly (it carries no OpenAPI surface); `RaftRole`,
//! `PeerAddr`, and `ClusterStateView` keep lumen-side `utoipa::ToSchema`
//! wrappers here — with `From<>` conversions to the `raft_host` shapes —
//! since deriving `ToSchema` on the shared types would force every adopter
//! (keep/relay/loom) to pull in utoipa whether or not it exposes an OpenAPI
//! doc.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::config::ClusterConfig;

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
pub use raft_host::ReadConsistency;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
pub enum RaftRole {
    Leader,
    Follower,
    Learner,
    Candidate,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
impl From<raft_host::RaftRole> for RaftRole {
    fn from(role: raft_host::RaftRole) -> Self {
        match role {
            raft_host::RaftRole::Leader => Self::Leader,
            raft_host::RaftRole::Follower => Self::Follower,
            raft_host::RaftRole::Learner => Self::Learner,
            raft_host::RaftRole::Candidate => Self::Candidate,
        }
    }
}

/// Peer list for one shard. The address scheme is the same for every
/// deployment: `lumen-{ordinal}.{headless_service}:{port}` where the
/// pod ordinal is `replica * shard_count + shard`.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
pub struct RaftGroup {
    pub shard_index: u32,
    pub peers: Vec<PeerAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
pub struct PeerAddr {
    pub pod_name: String,
    pub host: String,
    pub raft_port: u16,
    pub client_port: u16,
    pub role: RaftRole,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
impl From<raft_host::PeerAddr> for PeerAddr {
    fn from(p: raft_host::PeerAddr) -> Self {
        Self {
            pod_name: p.pod_name,
            host: p.host,
            raft_port: p.raft_port,
            client_port: p.client_port,
            role: p.role.into(),
        }
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
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
            // Pod-ordinal math shared with `raft_host::cluster::ClusterTopology`
            // (#1002) — no local `%`/`/` peer-DNS arithmetic.
            let ordinal = raft_host::cluster::peer_ordinal(cfg.shard_count, shard, replica);
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
        // single machine; index N maps to replica N in this shard. Parsing
        // is shared with `ClusterTopology::from_env`'s peer override (#1002).
        let overrides = raft_host::cluster::parse_peer_overrides("LUMEN_PEERS");
        for (i, peer) in peers.iter_mut().enumerate() {
            if let Some(addr) = overrides.get(i) {
                if let Some((host, port)) = addr.rsplit_once(':') {
                    peer.host = host.to_string();
                    peer.raft_port = port.parse().unwrap_or(peer.raft_port);
                } else {
                    peer.host = addr.clone();
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
/// in place from background replication tasks.
#[derive(Debug)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-raft-rs.md#source
impl From<raft_host::ClusterStateView> for ClusterStateView {
    fn from(v: raft_host::ClusterStateView) -> Self {
        Self {
            pod_name: v.pod_name,
            shard_index: v.shard_index,
            replica_index: v.replica_index,
            role: v.role.into(),
            peers: v.peers.into_iter().map(Into::into).collect(),
            applied_index: v.applied_index,
            leader_term: v.leader_term,
            replication_lag_ms: v.replication_lag_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_types_convert_from_raft_host_canonical_shape() {
        // The header contract itself is a pure re-export — no local wrapper —
        // so its parsing tests moved down to raft-host (#1003).
        for (host_role, local_role) in [
            (raft_host::RaftRole::Leader, RaftRole::Leader),
            (raft_host::RaftRole::Follower, RaftRole::Follower),
            (raft_host::RaftRole::Learner, RaftRole::Learner),
            (raft_host::RaftRole::Candidate, RaftRole::Candidate),
        ] {
            assert_eq!(RaftRole::from(host_role), local_role);
        }

        let host_peer = raft_host::PeerAddr {
            pod_name: "lumen-1".into(),
            host: "lumen-1.lumen-peer".into(),
            raft_port: 8082,
            client_port: 8080,
            role: raft_host::RaftRole::Follower,
        };
        let peer: PeerAddr = host_peer.clone().into();
        assert_eq!(peer.pod_name, host_peer.pod_name);
        assert_eq!(peer.role, RaftRole::Follower);

        let host_view = raft_host::ClusterStateView {
            pod_name: "lumen-1".into(),
            shard_index: 0,
            replica_index: 1,
            role: raft_host::RaftRole::Follower,
            peers: vec![host_peer],
            applied_index: 5,
            leader_term: 2,
            replication_lag_ms: 10,
        };
        let view: ClusterStateView = host_view.into();
        assert_eq!(view.pod_name, "lumen-1");
        assert_eq!(view.role, RaftRole::Follower);
        assert_eq!(view.peers.len(), 1);
        assert_eq!(view.applied_index, 5);
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
// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/raft.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/raft.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
