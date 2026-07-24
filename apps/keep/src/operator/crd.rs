//! The `Keep` custom resource (`keep.dev/v1alpha1`).
//!
//! One `Keep` object declares a keep deployment's HA topology. The spec flattens
//! the shared [`service_k8s::ClusterSpec`] (image + sharding/replication knobs +
//! per-pod resources) and adds keep's own runtime knobs (per-node engine shard
//! count, log level, drain window, and the per-pod durable disk tier). The
//! reconcile loop ([`super::reconcile`]) turns it into a ServiceAccount,
//! ConfigMap, headless + client Services, a PodDisruptionBudget, and the sharded
//! StatefulSet, garbage-collected via owner references.
//!
//! @spec .aw/tech-design/projects/keep/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `keep.dev/v1alpha1` `Keep`. Namespaced: every child object the operator
/// renders lands in this object's namespace, so independent keep deployments
/// coexist by name.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "keep.dev",
    version = "v1alpha1",
    kind = "Keep",
    plural = "keeps",
    shortname = "kp",
    namespaced,
    status = "KeepStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#,
    printcolumn = r#"{"name":"Shards","type":"integer","jsonPath":".status.shardCount"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct KeepSpec {
    /// The shared sharded-HA cluster shape — `image`, `imagePullPolicy`,
    /// `shardCount`, `replicasPerShard`, `voterCount`, and per-pod `resources`.
    /// Flattened so the CRD carries these fields directly (no `cluster:`
    /// nesting), exactly as the render toolkit expects.
    #[serde(flatten)]
    pub cluster: service_k8s::ClusterSpec,

    /// Per-node engine shard count (`KEEP_SHARDS`) for multi-core scaling —
    /// distinct from `shardCount` (the cluster-routing fan-out). Defaults to 256.
    #[serde(default = "default_engine_shards")]
    pub engine_shards: u32,

    /// Log level (`trace|debug|info|warn|error`). Defaults to `info`.
    #[serde(default)]
    pub log_level: Option<String>,

    /// Graceful drain window on SIGTERM (seconds); tracks
    /// `terminationGracePeriodSeconds`. Defaults to 30.
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,

    /// Per-pod durable disk-tier (WAL + snapshots) PVC size. Defaults to `10Gi`.
    #[serde(default = "default_storage")]
    pub storage: String,

    /// PVC StorageClass for the disk tier. Unset means the cluster default.
    #[serde(default)]
    pub storage_class: Option<String>,

    /// Optional scheduled backup (#776). When set, the operator renders a backup
    /// CronJob (see [`super::render`]) that invokes `keep backup` on this
    /// schedule to the given destination, applying the retention. Absent means
    /// no backup CronJob.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<KeepBackupSpec>,

    /// Max request body size in bytes (#2556, `KEEP_BODY_LIMIT`). Bounds
    /// claim-check blob writes. Defaults to 16 MiB if unset. Must be a positive
    /// integer.
    #[serde(default = "default_body_limit_bytes")]
    pub body_limit_bytes: u64,
}

/// Declarative backup policy carried on a `Keep` CR (#776).
///
/// Keep adds no app-specific fields, so its public Rust name is a direct alias
/// of the shared flat structural-schema-safe contract.
pub use service_backup::ScheduledBackupPolicy as KeepBackupSpec;

/// Status subresource, written back by the reconcile loop.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KeepStatus {
    /// `Pending | Reconciling | Ready`.
    #[serde(default)]
    pub phase: String,
    /// The `.metadata.generation` this status reflects (drift detection).
    #[serde(default)]
    pub observed_generation: i64,
    /// Ready store replicas (from the StatefulSet status).
    #[serde(default)]
    pub ready_replicas: i32,
    /// Desired store replicas (`shardCount * replicasPerShard`).
    #[serde(default)]
    pub desired_replicas: i32,
    /// Effective cluster shard count.
    #[serde(default)]
    pub shard_count: u32,
    /// Last human-readable reconcile message.
    #[serde(default)]
    pub message: String,
}

fn default_engine_shards() -> u32 {
    256
}
fn default_grace_secs() -> u64 {
    30
}
fn default_storage() -> String {
    "10Gi".into()
}
fn default_body_limit_bytes() -> u64 {
    16 * 1024 * 1024 // 16 MiB, matching keep::http::DEFAULT_BODY_LIMIT
}
