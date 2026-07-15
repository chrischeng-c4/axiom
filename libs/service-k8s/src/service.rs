// SPEC-MANAGED: libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The [`ManagedService`] trait a service implements + the shared CRD fragments.

use std::collections::HashMap;
use std::fmt::Debug;

use kube::core::NamespaceResourceScope;
use kube::{CustomResourceExt, Resource};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A workload to poll for `.status.readyReplicas` during reconcile.
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ReadinessTarget {
    pub kind: &'static str,
    pub name: String,
}

/// Observed readiness handed to [`ManagedService::status_patch`]
/// (workload name → `readyReplicas`).
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ReadyFacts {
    pub ready: HashMap<String, i64>,
}

/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
impl ReadyFacts {
    /// Ready replicas for `name`, or 0 if the workload was absent.
    pub fn get(&self, name: &str) -> i64 {
        self.ready.get(name).copied().unwrap_or(0)
    }
}

/// One service's contribution to the shared operator. Implemented on the CRD
/// root type (e.g. lumen's `Lumen`). The [`crate::controller`] is generic over
/// `S`, so the watch/apply/lease loop is written once.
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
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
/// `#[serde(flatten)] pub cluster: service_k8s::ClusterSpec`.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ClusterSpec {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<String>,
    #[serde(default = "one")]
    pub shard_count: u32,
    /// Starting/minimum members per shard. With startup-static raft-runtime
    /// membership this is also the fixed desired value; a future membership
    /// controller may plan whole replica layers above this floor.
    #[serde(default = "one")]
    pub replicas_per_shard: u32,
    #[serde(default = "one")]
    pub voter_count: u32,
    #[serde(default)]
    pub resources: ResourceSpec,
}

/// Per-pod CPU/memory requests. Empty values resolve to the shared data-plane
/// defaults (`1` CPU / `4Gi`) at render time. Limits are intentionally omitted
/// so a dedicated-node pod can use otherwise-idle node capacity.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec libs/service-k8s/tech-design/semantic/source/libs-service-k8s-src-service-rs.md#source
pub struct ResourceSpec {
    #[serde(default)]
    pub cpu: String,
    #[serde(default)]
    pub memory: String,
}

fn one() -> u32 {
    1
}
// CODEGEN-END
