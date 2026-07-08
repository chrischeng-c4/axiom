// SPEC-MANAGED: libs/raft-host/tech-design/semantic/source/libs-raft-host-src-view-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Cluster-introspection view model.
//!
//! The shape every raft_core service's cluster-introspection endpoint (e.g.
//! lumen's `/debug/cluster`) reports: this node's role, its peer group, and
//! basic replication telemetry. Kept dependency-free (no `utoipa`) so an
//! adopter that doesn't expose an OpenAPI doc isn't forced to pull one in; a
//! caller that does (lumen) keeps a local `utoipa::ToSchema` wrapper mapping
//! `From<>` these canonical shapes instead.

use serde::{Deserialize, Serialize};

/// A raft group member's role, as observed by cluster introspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-view-rs.md#source
pub enum RaftRole {
    Leader,
    Follower,
    Learner,
    Candidate,
}

/// One peer's address + role within a raft group.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-view-rs.md#source
pub struct PeerAddr {
    pub pod_name: String,
    pub host: String,
    pub raft_port: u16,
    pub client_port: u16,
    pub role: RaftRole,
}

/// Live cluster snapshot — the wire shape of a cluster-introspection view.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec libs/raft-host/tech-design/semantic/source/libs-raft-host-src-view-rs.md#source
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
// CODEGEN-END
