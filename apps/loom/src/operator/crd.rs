//! The `Loom` custom resource (`loom.dev/v1alpha1`).
//!
//! One `Loom` object declares a full control-plane deployment. Single-replica
//! instances run single-node raft; multi-replica instances run a raft HA group
//! via a StatefulSet (stable peer identity + the downward-API env
//! `raft_host::ClusterTopology::from_env` reads). The reconcile loop in
//! [`super::reconcile`] turns this spec into ServiceAccount, StatefulSet,
//! Services (headless + client), and PDB objects, GC'd via owner references.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn one() -> u32 {
    1
}
fn default_relay() -> String {
    "http://relay.relay.svc.cluster.local:7400".to_string()
}
fn default_keep() -> String {
    "http://keep.keep.svc.cluster.local:7117".to_string()
}
fn default_completion_shards() -> u32 {
    8
}
fn default_gc_retention() -> u64 {
    3600
}
fn default_storage() -> String {
    "5Gi".to_string()
}
fn default_cpu() -> String {
    "500m".to_string()
}
fn default_memory() -> String {
    "512Mi".to_string()
}

/// `loom.dev/v1alpha1` `Loom`. Namespaced: every child object the operator
/// renders lands in this object's namespace, so multiple independent loom
/// deployments can coexist by name.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "loom.dev",
    version = "v1alpha1",
    kind = "Loom",
    plural = "looms",
    shortname = "lm",
    namespaced,
    status = "LoomStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#,
    printcolumn = r#"{"name":"Voters","type":"integer","jsonPath":".spec.voterCount"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct LoomSpec {
    /// Controller container image (all roles run from it).
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<String>,
    /// Raft topology. `replicasPerShard > 1` turns on the raft HA group.
    #[serde(default = "one")]
    pub shard_count: u32,
    #[serde(default = "one")]
    pub replicas_per_shard: u32,
    #[serde(default = "one")]
    pub voter_count: u32,
    /// relay broker base URL (loom → relay dispatch).
    #[serde(default = "default_relay")]
    pub relay: String,
    /// keep store base URL (claim-check payloads).
    #[serde(default = "default_keep")]
    pub keep: String,
    /// Completion-consumer shard count (must match the workers' sink).
    #[serde(default = "default_completion_shards")]
    pub completion_shards: u32,
    /// Completed-DAG GC retention window (seconds); 0 disables.
    #[serde(default = "default_gc_retention")]
    pub gc_retention_secs: u64,
    /// Per-pod raft data volume size.
    #[serde(default = "default_storage")]
    pub storage: String,
    #[serde(default = "default_cpu")]
    pub cpu: String,
    #[serde(default = "default_memory")]
    pub memory: String,
    /// Cron schedule for the raft-snapshot backup CronJob (omit to disable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_schedule: Option<String>,
    /// Backup destination URI (`file:///path`, `s3://bucket/prefix`,
    /// `gs://bucket/prefix`); required when `backupSchedule` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup_destination: Option<String>,
}

/// The `Loom` `.status` subresource the operator writes each reconcile.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoomStatus {
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub observed_generation: i64,
    #[serde(default)]
    pub ready_replicas: i32,
    #[serde(default)]
    pub desired_replicas: i32,
    #[serde(default)]
    pub shard_count: u32,
    #[serde(default)]
    pub message: String,
}
