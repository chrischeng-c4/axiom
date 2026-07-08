---
id: libs-raft-host-src-view-rs
summary: Lossless rust-source-unit coverage for `libs/raft-host/src/view.rs`.
capability_refs:
  - id: shared-raft-host-driver
    role: primary
    claim: shared-raft-host-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/raft-host/src/view.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/raft-host/src/view.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RaftRole` | libs/raft-host/src/view.rs | enum | pub | 15 | pub enum RaftRole { |
| `PeerAddr` | libs/raft-host/src/view.rs | struct | pub | 24 | pub struct PeerAddr { |
| `ClusterStateView` | libs/raft-host/src/view.rs | struct | pub | 34 | pub struct ClusterStateView { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
pub enum RaftRole {
    Leader,
    Follower,
    Learner,
    Candidate,
}

/// One peer's address + role within a raft group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddr {
    pub pod_name: String,
    pub host: String,
    pub raft_port: u16,
    pub client_port: u16,
    pub role: RaftRole,
}

/// Live cluster snapshot — the wire shape of a cluster-introspection view.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/raft-host/src/view.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/raft-host/src/view.rs` captured during libs codegen standardization.
```
