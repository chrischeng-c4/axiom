// HANDWRITE-BEGIN gap="missing-generator:logic:bfdc7475" tracker="pending-tracker" reason="TapeSpec CustomResource (group tape.dev, v1alpha1, kind Tape, plural tapes, shortname tp, namespaced, status TapeStatus, printcolumns Phase/Ready/Age): #[serde(flatten)] cluster: service_k8s::ClusterSpec (shardCount defaults 1, pinned by the render -- tape is a single raft group) + storage (default 10Gi) + storageClass + graceSecs (default 10) + logLevel (Option) + auth (flat string off|required) + tokensSecret (Option<String>). TapeStatus { phase, observedGeneration, readyReplicas, desiredReplicas, message }."
//! The `Tape` custom resource (`tape.dev/v1alpha1`).
//!
//! One `Tape` object declares a tape deployment's HA topology. The spec
//! flattens the shared [`service_k8s::ClusterSpec`] (image + sharding/replication
//! knobs + per-pod resources) and adds tape's own runtime knobs (durable
//! journal disk tier, drain window, log level, and the opt-in bearer-auth
//! wiring). tape is a **single raft group**: `shardCount` exists in the
//! shared shape but defaults to 1 and the render pins it —
//! `replicasPerShard` is the only scale knob (1 = single node, 3 = raft HA).
//!
//! HA peer DNS note: tape's serve path derives peer addresses from the
//! `TAPE_PEER_SERVICE` headless Service (`raft_runtime::cluster::ClusterTopology`
//! resolves per-pod peer hosts from the downward-API quartet), so an HA
//! (`replicasPerShard > 1`) instance's headless Service must match.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `tape.dev/v1alpha1` `Tape`. Namespaced: every child object the operator
/// renders lands in this object's namespace, so independent tape deployments
/// coexist by name.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "tape.dev",
    version = "v1alpha1",
    kind = "Tape",
    plural = "tapes",
    shortname = "tp",
    namespaced,
    status = "TapeStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct TapeSpec {
    /// The shared sharded-HA cluster shape — `image`, `imagePullPolicy`,
    /// `shardCount`, `replicasPerShard`, `voterCount`, and per-pod
    /// `resources`. Flattened so the CRD carries these fields directly (no
    /// `cluster:` nesting), exactly as the render toolkit expects. tape is a
    /// single raft group: the render pins `shardCount` to 1.
    #[serde(flatten)]
    pub cluster: service_k8s::ClusterSpec,

    /// Per-pod journal plus shared Raft hard-state/log/snapshot PVC size.
    /// Defaults to `10Gi`.
    #[serde(default = "default_storage")]
    pub storage: String,

    /// PVC StorageClass for the journal disk. Unset means the cluster
    /// default.
    #[serde(default)]
    pub storage_class: Option<String>,

    /// Graceful drain window on SIGTERM (seconds); tracks
    /// `terminationGracePeriodSeconds`. Defaults to 10 (`TAPE_GRACE_SECS`).
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,

    /// Log level (`trace|debug|info|warn|error`), injected as `RUST_LOG`.
    /// Unset means the server default (`info`).
    #[serde(default)]
    pub log_level: Option<String>,

    /// Request-auth mode for the data plane: `off` (default) or `required`.
    /// A flat string, not an enum with divergent variant schemas
    /// (Kubernetes structural schemas cannot represent those). `required`
    /// takes effect only together with [`Self::tokens_secret`].
    #[serde(default = "default_auth")]
    pub auth: String,

    /// Name of the Secret carrying the bearer-token registry (key
    /// `token-registry.json`). When set with `auth: required`, the render
    /// mounts it read-only at `/var/run/secrets/tape` and injects
    /// `TAPE_AUTH=required` + `TAPE_TOKEN_REGISTRY_FILE` (relay/lumen's
    /// pattern; off unless the CR asks). Tape watches a changed projection
    /// and atomically applies only a valid replacement without a pod restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret: Option<String>,

    /// Name of a Secrets Store CSI `SecretProviderClass` that projects the
    /// same `token-registry.json` file as [`Self::tokens_secret`]. When the
    /// CSI driver refreshes that file, Tape's watcher applies valid rotations
    /// without a pod restart. It is used only with `auth: required`; when both
    /// sources are set, `tokensSecret` wins for backward compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret_provider_class: Option<String>,

    /// Exact backup object URI used only when a replacement replica starts on
    /// a fresh PVC. The server consumes it before Raft catch-up and refuses a
    /// non-empty data directory, so this is cold recovery rather than live
    /// replica synchronization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_seed_uri: Option<String>,
}

/// Status subresource, written back by the reconcile loop.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TapeStatus {
    /// `Pending | Reconciling | Ready`.
    #[serde(default)]
    pub phase: String,
    /// The `.metadata.generation` this status reflects (drift detection).
    #[serde(default)]
    pub observed_generation: i64,
    /// Ready replicas (from the StatefulSet status).
    #[serde(default)]
    pub ready_replicas: i32,
    /// Desired replicas (`replicasPerShard`; tape is single-group).
    #[serde(default)]
    pub desired_replicas: i32,
    /// Last human-readable reconcile message.
    #[serde(default)]
    pub message: String,
}

fn default_storage() -> String {
    "10Gi".into()
}
fn default_grace_secs() -> u64 {
    10
}
fn default_auth() -> String {
    "off".into()
}
// HANDWRITE-END
