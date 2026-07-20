// HANDWRITE-BEGIN gap="missing-generator:logic:07a338e2" tracker="pending-tracker" reason="RelaySpec CustomResource (group relay.dev, v1alpha1, kind Relay, plural relays, namespaced, status RelayStatus, printcolumns Phase/Ready/Age): #[serde(flatten)] cluster: service_k8s::ClusterSpec (shardCount defaults 1; relay is a single raft group — render pins it) + storage (default 10Gi), storageClass, graceSecs (default 10), logLevel (Option, RUST_LOG), auth (flat string off|required — no divergent-variant enums in the CRD), tokensSecret (Option<String> Secret name); RelayStatus { phase, observedGeneration, readyReplicas, desiredReplicas, message }."
//! The `Relay` custom resource (`relay.dev/v1alpha1`).
//!
//! One `Relay` object declares a relay deployment's HA topology. The spec
//! flattens the shared [`service_k8s::ClusterSpec`] (image + sharding/replication
//! knobs + per-pod resources) and adds relay's own runtime knobs (durable-log
//! disk tier, drain window, log level, and the opt-in bearer-auth wiring).
//! relay is a **single raft group**: `shardCount` exists in the shared shape
//! but defaults to 1 and the render pins it — `replicasPerShard` is the only
//! scale knob (1 = single node, 3 = raft HA).
//!
//! HA peer DNS note: relay's serve path derives peer addresses as
//! `relay-<ordinal>.<RELAY_PEER_SERVICE>` (the `relay` pod prefix is fixed in
//! the serve contract), so an HA (`replicasPerShard > 1`) instance must be
//! named `relay`; `RELAY_PEERS` overrides peer DNS for local groups.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `relay.dev/v1alpha1` `Relay`. Namespaced: every child object the operator
/// renders lands in this object's namespace, so independent relay deployments
/// coexist by name.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "relay.dev",
    version = "v1alpha1",
    kind = "Relay",
    plural = "relays",
    shortname = "rly",
    namespaced,
    status = "RelayStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct RelaySpec {
    /// The shared sharded-HA cluster shape — `image`, `imagePullPolicy`,
    /// `shardCount`, `replicasPerShard`, `voterCount`, and per-pod
    /// `resources`. Flattened so the CRD carries these fields directly (no
    /// `cluster:` nesting), exactly as the render toolkit expects. relay is a
    /// single raft group: the render pins `shardCount` to 1.
    #[serde(flatten)]
    pub cluster: service_k8s::ClusterSpec,

    /// Per-pod durable-log (+ raft hard state) PVC size. Defaults to `10Gi`.
    #[serde(default = "default_storage")]
    pub storage: String,

    /// PVC StorageClass for the durable log. Unset means the cluster default.
    #[serde(default)]
    pub storage_class: Option<String>,

    /// Graceful drain window on SIGTERM (seconds); tracks
    /// `terminationGracePeriodSeconds`. Defaults to 10 (`RELAY_GRACE_SECS`).
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,

    /// Log level (`trace|debug|info|warn|error`), injected as `RUST_LOG`.
    /// Unset means the server default (`info`).
    #[serde(default)]
    pub log_level: Option<String>,

    /// Request-auth mode for the /v1 data plane: `off` (default) or
    /// `required`. A flat string, not an enum with divergent variant schemas
    /// (Kubernetes structural schemas cannot represent those — keep #776).
    /// `required` takes effect only together with [`Self::tokens_secret`].
    #[serde(default = "default_auth")]
    pub auth: String,

    /// Name of the Secret carrying the bearer-token registry (key
    /// `token-registry.json`). When set with `auth: required`, the render
    /// mounts it read-only at `/var/run/secrets/relay` and injects
    /// `RELAY_AUTH=required` + `RELAY_TOKEN_REGISTRY_FILE` (lumen's pattern;
    /// off unless the CR asks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret: Option<String>,

    /// Secret containing `tls.crt`, `tls.key`, and `ca.crt` for authenticated
    /// Raft peer traffic on the dedicated 7001 listener. Omit only for local
    /// cleartext development clusters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_tls_secret: Option<String>,

    /// Optional scheduled backup (#1209). When set, the operator renders a
    /// `<name>-backup` CronJob (see [`super::render`]) invoking `relay
    /// backup` on this schedule against the deployment's own
    /// `GET /admin/backup` endpoint — no new snapshot mechanism, only
    /// scheduling + transport (lumen #808). Absent means no CronJob; the
    /// endpoint stays reachable for manual/scripted use either way.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<RelayBackupSpec>,
}

/// Declarative backup policy (#1209).
///
/// Common CRD-safe fields come from
/// [`service_backup::ScheduledBackupPolicy`]. Relay owns only the optional
/// admin-token Secret reference used by its runner CronJob.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelayBackupSpec {
    /// Shared flat `schedule`, `destination`, and `retentionSecs` contract.
    #[serde(flatten)]
    pub policy: service_backup::ScheduledBackupPolicy,
    /// Name of a Secret whose `token` key holds a bearer token with `admin`
    /// on `*`, injected into the CronJob as `RELAY_BACKUP_TOKEN`. Needed when
    /// `auth: required`; ignored (the endpoint needs no token) when auth is
    /// off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token_secret: Option<String>,
}

impl std::ops::Deref for RelayBackupSpec {
    type Target = service_backup::ScheduledBackupPolicy;

    fn deref(&self) -> &Self::Target {
        &self.policy
    }
}

/// Status subresource, written back by the reconcile loop.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelayStatus {
    /// `Pending | Reconciling | Ready`.
    #[serde(default)]
    pub phase: String,
    /// The `.metadata.generation` this status reflects (drift detection).
    #[serde(default)]
    pub observed_generation: i64,
    /// Ready broker replicas (from the StatefulSet status).
    #[serde(default)]
    pub ready_replicas: i32,
    /// Desired broker replicas (`replicasPerShard`; relay is single-group).
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
