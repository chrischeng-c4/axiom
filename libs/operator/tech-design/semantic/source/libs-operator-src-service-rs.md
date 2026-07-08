---
id: libs-operator-src-service-rs
summary: Lossless rust-source-unit coverage for `libs/operator/src/service.rs`.
capability_refs:
  - id: shared-kubernetes-operator-scaffold
    role: primary
    claim: shared-kubernetes-operator-scaffold-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Operator library contract."
fill_sections: [overview, source, changes]
---

# Standardized libs/operator/src/service.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/operator/src/service.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReadinessTarget` | libs/operator/src/service.rs | struct | pub | 13 | pub struct ReadinessTarget { |
| `ReadyFacts` | libs/operator/src/service.rs | struct | pub | 20 | pub struct ReadyFacts { |
| `get` | libs/operator/src/service.rs | function | pub | 26 | pub fn get(&self, name: &str) -> i64 { |
| `ManagedService` | libs/operator/src/service.rs | trait | pub | 34 | pub trait ManagedService: |
| `ClusterSpec` | libs/operator/src/service.rs | struct | pub | 63 | pub struct ClusterSpec { |
| `ResourceSpec` | libs/operator/src/service.rs | struct | pub | 80 | pub struct ResourceSpec { |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! The [`ManagedService`] trait a service implements + the shared CRD fragments.

use std::collections::HashMap;
use std::fmt::Debug;

use kube::core::NamespaceResourceScope;
use kube::{CustomResourceExt, Resource};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A workload to poll for `.status.readyReplicas` during reconcile.
pub struct ReadinessTarget {
    pub kind: &'static str,
    pub name: String,
}

/// Observed readiness handed to [`ManagedService::status_patch`]
/// (workload name → `readyReplicas`).
pub struct ReadyFacts {
    pub ready: HashMap<String, i64>,
}

impl ReadyFacts {
    /// Ready replicas for `name`, or 0 if the workload was absent.
    pub fn get(&self, name: &str) -> i64 {
        self.ready.get(name).copied().unwrap_or(0)
    }
}

/// One service's contribution to the shared operator. Implemented on the CRD
/// root type (e.g. lumen's `Lumen`). The [`crate::controller`] is generic over
/// `S`, so the watch/apply/lease loop is written once.
pub trait ManagedService:
    Resource<DynamicType = (), Scope = NamespaceResourceScope>
    + CustomResourceExt
    + Clone
    + Debug
    + DeserializeOwned
    + Send
    + Sync
    + 'static
{
    /// Server-side-apply field manager **and** the leader-election Lease name.
    /// Per-service so two operators never collide on the same Lease.
    const MANAGER: &'static str;

    /// Pure render: the spec (+ metadata via `ResourceExt`) → the child objects
    /// to server-side-apply. No I/O.
    fn render(&self) -> Vec<serde_json::Value>;

    /// The workloads whose `.status.readyReplicas` feed [`Self::status_patch`].
    fn readiness_targets(&self) -> Vec<ReadinessTarget>;

    /// The `{ "status": { … } }` subresource patch given observed readiness.
    fn status_patch(&self, ready: &ReadyFacts) -> serde_json::Value;
}

/// The generic cluster shape every sharded-HA service embeds in its CRD spec via
/// `#[serde(flatten)] pub cluster: operator::ClusterSpec`.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSpec {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<String>,
    #[serde(default = "one")]
    pub shard_count: u32,
    #[serde(default = "one")]
    pub replicas_per_shard: u32,
    #[serde(default = "one")]
    pub voter_count: u32,
    #[serde(default)]
    pub resources: ResourceSpec,
}

/// CPU/memory request==limit (Guaranteed QoS).
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpec {
    #[serde(default)]
    pub cpu: String,
    #[serde(default)]
    pub memory: String,
}

fn one() -> u32 {
    1
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/operator/src/service.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/operator/src/service.rs` captured during libs codegen standardization.
```
