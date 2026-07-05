// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The `Lumen` custom resource (`lumen.dev/v1alpha1`).
//!
//! One `Lumen` object declares a full deployment. Single-replica instances
//! write to a local WAL with no raft consensus; multi-replica instances add
//! Lumen-owned raft replication on top. Both regimes render the serving fleet
//! as a StatefulSet with a durable per-pod `raft` PVC backing the WAL —
//! `replicasPerShard` only gates raft consensus, never persistence. The
//! reconcile loop in [`super::reconcile`] turns this spec into StatefulSet,
//! Service, ConfigMap, HPA (single-member regime only), PDB, and
//! ServiceAccount objects, garbage-collected via owner references.

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `lumen.dev/v1alpha1` `Lumen`. Namespaced: every child object the operator
/// renders lands in this object's namespace, so multiple independent lumen
/// deployments can coexist by name.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "lumen.dev",
    version = "v1alpha1",
    kind = "Lumen",
    plural = "lumens",
    shortname = "lum",
    namespaced,
    status = "LumenStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.servingReadyReplicas"}"#,
    printcolumn = r#"{"name":"Shards","type":"integer","jsonPath":".status.shardCount"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct LumenSpec {
    /// Serving container image, e.g. `lumen:latest`. Required.
    pub image: String,

    /// Image pull policy. Defaults to `IfNotPresent`.
    #[serde(default)]
    pub image_pull_policy: Option<String>,

    /// Install-time shard fan-out for client-side `crc32(collection) % N`
    /// routing. The operator surfaces it but never reshards a live cluster
    /// (clients own the routing); treat as immutable after first apply.
    #[serde(default = "default_shard_count")]
    pub shard_count: u32,

    /// Raft replicas per shard. `1` (default) = a single-member serving
    /// StatefulSet with no raft consensus (still durable — the same
    /// PVC-backed `raft` volume — and still fronted by an HPA). `> 1` adds
    /// raft-HA: a fixed peer set whose pods inject the downward-API env
    /// `raft_host::cluster` reads (no HPA — raft needs a known membership).
    #[serde(default = "default_replicas_per_shard")]
    pub replicas_per_shard: u32,

    /// Voting members per shard (the rest are learners). Only meaningful when
    /// `replicasPerShard > 1`.
    #[serde(default = "default_replicas_per_shard")]
    pub voter_count: u32,

    /// Log output format: `json` (prod/staging) or `pretty` (dev).
    #[serde(default)]
    pub log_format: LogFormat,

    /// Log level (`trace|debug|info|warn|error`). Defaults to `info`.
    #[serde(default)]
    pub log_level: Option<String>,

    /// Auth mode: `off` (dev) or `required` (token registry supplied via
    /// `tokensSecret`).
    #[serde(default)]
    pub auth: AuthMode,

    /// Name of a Secret whose `token-registry.json` key is mounted at
    /// `/var/run/secrets/lumen/token-registry.json` and exposed to the serving
    /// process as `LUMEN_TOKEN_REGISTRY_FILE` when `auth: required`.
    /// `token-registry.json` is a JSON object of
    /// `{ "<token>": { "subject": "...", "roles": { "<collection_id>|*": "read|write|admin" } } }`.
    /// Ignored when `auth: off`.
    #[serde(default)]
    pub tokens_secret: Option<String>,

    /// Stateless serving-fleet shape.
    #[serde(default)]
    pub serving: ServingSpec,

    /// Emit a ServiceMonitor + PrometheusRule. Requires the prometheus-operator
    /// CRDs (`monitoring.coreos.com/v1`) to be installed in the cluster.
    #[serde(default)]
    pub observability: bool,
}

/// Log output format.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum LogFormat {
    /// Structured one-line-per-event JSON (prod/staging).
    Json,
    /// Human-readable multi-line (dev).
    #[default]
    Pretty,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl LogFormat {
    /// The `LUMEN_LOG_FORMAT` value the serving binary expects.
    pub fn as_env(self) -> &'static str {
        match self {
            LogFormat::Json => "json",
            LogFormat::Pretty => "pretty",
        }
    }
}

/// Whether the client API requires a bearer token.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum AuthMode {
    /// Open API (dev / trusted network). Serialized as `disabled` — NOT `off`,
    /// which YAML 1.1 (kubectl / go-yaml) would parse as the boolean `false`
    /// and corrupt the CRD enum/default.
    #[default]
    #[serde(rename = "disabled")]
    Off,
    /// Bearer-token required; the token registry file comes from `tokensSecret`.
    Required,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl AuthMode {
    /// The `LUMEN_AUTH` value the serving binary expects.
    pub fn as_env(self) -> &'static str {
        match self {
            AuthMode::Off => "off",
            AuthMode::Required => "required",
        }
    }
}

/// Stateless serving-fleet shape: autoscaling bounds + per-pod resources.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ServingSpec {
    /// HPA bounds + CPU target.
    #[serde(default)]
    pub autoscaling: Autoscaling,
    /// Per-pod CPU, applied as request==limit (Guaranteed QoS). e.g. `"2"`.
    #[serde(default = "default_serving_cpu")]
    pub cpu: String,
    /// Per-pod memory, applied as request==limit. e.g. `"4Gi"`.
    #[serde(default = "default_serving_memory")]
    pub memory: String,
    /// Graceful drain window on SIGTERM (seconds); tracks
    /// `terminationGracePeriodSeconds`.
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,
    /// Per-pod WAL/raft hard-state PVC size. Always applied — the serving
    /// StatefulSet's `raft` volumeClaimTemplate exists at every
    /// `replicasPerShard` value, not only when raft consensus (`> 1`) is
    /// active.
    #[serde(default = "default_raft_storage")]
    pub raft_storage: String,
    /// PVC StorageClass for the WAL/raft hard-state volume. Unset means
    /// cluster default — which, on most managed Kubernetes offerings, is
    /// **not** SSD-backed (e.g. GKE's default `standard-rwo` is
    /// pd-balanced, not pd-ssd). Raft/WAL write latency is sensitive to
    /// disk performance, so a deployer who cares about that latency should
    /// set this field explicitly to an SSD-backed StorageClass name rather
    /// than relying on the cluster default (see `lumen llm --topic storage` for
    /// example StorageClass names per common provider — informational
    /// reference only, not a value validated or defaulted by this field).
    #[serde(default)]
    pub raft_storage_class: Option<String>,
    /// Optional scheduled backup (#808). When set, the operator renders a
    /// `<name>-backup` CronJob (see [`super::render::backup_cron_job`]) that
    /// invokes `lumen backup` on this schedule against the running serving
    /// fleet's own already-existing `/admin/backup` endpoint — no new
    /// snapshot mechanism, only scheduling + transport. Absent means no
    /// CronJob; the admin API (`GET /admin/backup`, `POST /admin/backup/local`,
    /// `POST /admin/restore`) is still reachable for manual/scripted use
    /// either way (see `lumen llm --topic storage`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<ServingBackupSpec>,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for ServingSpec {
    fn default() -> Self {
        Self {
            autoscaling: Autoscaling::default(),
            cpu: default_serving_cpu(),
            memory: default_serving_memory(),
            grace_secs: default_grace_secs(),
            raft_storage: default_raft_storage(),
            raft_storage_class: None,
            backup: None,
        }
    }
}

/// Declarative backup policy for the serving fleet (#808).
///
/// The runner contract lives in `libs/service-backup`
/// (`BackupDestination`/`BackupSink`/`run_backup_once`); `lumen backup` parses
/// `destination` back into a `service_backup::BackupDestination` via
/// `from_uri`. This CRD-facing shape carries the destination as a URI string
/// (rather than the shared tagged-union `BackupDestination` schema, which
/// Kubernetes structural schemas cannot represent — a `prefix` property
/// shared across variants), mirroring keep's `KeepBackupSpec` (#776).
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ServingBackupSpec {
    /// Cron schedule (`CronJob.spec.schedule`) for the backup runner.
    pub schedule: String,
    /// Destination URI: `file:///path`, `s3://bucket/prefix`, or
    /// schema-only `gs://bucket/prefix` (parsed, but the runner supports
    /// `file://` and `s3://` sinks today).
    pub destination: String,
    /// Drop backup objects older than this many seconds after a successful
    /// put. Absent keeps everything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retention_secs: Option<u64>,
    /// Name of a Secret whose `token` key holds a bearer token with
    /// `Role::Admin` on `*`, injected into the CronJob as `LUMEN_BACKUP_TOKEN`.
    /// Needed when `spec.auth: required`; ignored (the admin API needs no
    /// token) when `spec.auth: off`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token_secret: Option<String>,
}

/// HPA bounds for the serving fleet.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct Autoscaling {
    /// Floor (also the StatefulSet's apply-time replica count in HPA mode).
    pub min_replicas: i32,
    /// Ceiling.
    pub max_replicas: i32,
    /// Target average CPU utilization (%) — read QPS proxied by CPU.
    pub target_cpu_utilization: i32,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for Autoscaling {
    fn default() -> Self {
        Self {
            min_replicas: 3,
            max_replicas: 12,
            target_cpu_utilization: 70,
        }
    }
}

/// Status subresource, written back by the reconcile loop.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct LumenStatus {
    /// `Pending | Reconciling | Ready | Degraded`.
    #[serde(default)]
    pub phase: String,
    /// The `.metadata.generation` this status reflects (drift detection).
    #[serde(default)]
    pub observed_generation: i64,
    /// Ready serving replicas (from the StatefulSet status).
    #[serde(default)]
    pub serving_ready_replicas: i32,
    /// Desired serving replicas (HPA floor at apply, or the live count).
    #[serde(default)]
    pub desired_replicas: i32,
    /// Effective shard count.
    #[serde(default)]
    pub shard_count: u32,
    /// Last human-readable reconcile message.
    #[serde(default)]
    pub message: String,
}

fn default_shard_count() -> u32 {
    1
}
fn default_replicas_per_shard() -> u32 {
    1
}
fn default_serving_cpu() -> String {
    "2".into()
}
fn default_serving_memory() -> String {
    "4Gi".into()
}
fn default_grace_secs() -> u64 {
    30
}
fn default_raft_storage() -> String {
    "20Gi".into()
}
// CODEGEN-END
