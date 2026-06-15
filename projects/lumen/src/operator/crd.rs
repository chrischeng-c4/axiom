// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The `Lumen` custom resource (`lumen.dev/v1alpha1`).
//!
//! One `Lumen` object declares a full deployment: a stateless, autoscaled
//! serving fleet plus a NATS JetStream write-log broker. The reconcile loop in
//! [`super::reconcile`] turns this spec into the same set of objects that
//! `k8s/base` + the overlays express by hand today — Deployment, Service,
//! ConfigMap, HPA, PDB, ServiceAccount (serving) and StatefulSet, Services,
//! ConfigMap (NATS) — but driven declaratively and garbage-collected via owner
//! references.

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
    /// Serving + (managed) NATS-sidecar-free container image, e.g.
    /// `lumen:latest`. Required.
    pub image: String,

    /// Image pull policy. Defaults to `IfNotPresent`.
    #[serde(default)]
    pub image_pull_policy: Option<String>,

    /// Install-time shard fan-out for client-side `crc32(collection) % N`
    /// routing. The operator surfaces it but never reshards a live cluster
    /// (clients own the routing); treat as immutable after first apply.
    #[serde(default = "default_shard_count")]
    pub shard_count: u32,

    /// Log output format: `json` (prod/staging) or `pretty` (dev).
    #[serde(default)]
    pub log_format: LogFormat,

    /// Log level (`trace|debug|info|warn|error`). Defaults to `info`.
    #[serde(default)]
    pub log_level: Option<String>,

    /// Auth mode: `off` (dev) or `required` (tokens supplied via
    /// `tokensSecret`).
    #[serde(default)]
    pub auth: AuthMode,

    /// Name of a Secret (key `LUMEN_TOKENS`) wired into serving pods when
    /// `auth: required`. Ignored when `auth: off`.
    #[serde(default)]
    pub tokens_secret: Option<String>,

    /// Stateless serving-fleet shape.
    #[serde(default)]
    pub serving: ServingSpec,

    /// NATS write-log broker. Managed by default; set `externalUrl` to point
    /// at an existing broker and skip provisioning one.
    #[serde(default)]
    pub nats: NatsSpec,

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
    /// Bearer-token required; tokens come from `tokensSecret`.
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
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for ServingSpec {
    fn default() -> Self {
        Self {
            autoscaling: Autoscaling::default(),
            cpu: default_serving_cpu(),
            memory: default_serving_memory(),
            grace_secs: default_grace_secs(),
        }
    }
}

/// HPA bounds for the serving fleet.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct Autoscaling {
    /// Floor (also the Deployment's apply-time replica count).
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

/// NATS write-log broker: either managed (StatefulSet) or external (BYO).
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct NatsSpec {
    /// Point serving pods at an existing broker. When set, the operator
    /// renders NO NATS objects (StatefulSet/Services/ConfigMap) and wires
    /// `LUMEN_NATS_URL` straight to this value.
    #[serde(default)]
    pub external_url: Option<String>,
    /// Managed-broker replica count. `1` (dev/staging) or `3+` for an HA
    /// JetStream cluster — the operator wires `--routes` when `> 1`.
    #[serde(default = "default_nats_replicas")]
    pub replicas: i32,
    /// Per-broker PVC size. e.g. `"20Gi"`, `"100Gi"`.
    #[serde(default = "default_nats_storage")]
    pub storage: String,
    /// PVC StorageClass. Unset → cluster default (portable; kind → local-path
    /// `standard`). Set to e.g. `"ssd"` in cloud.
    #[serde(default)]
    pub storage_class: Option<String>,
    /// Per-broker CPU, applied as request==limit.
    #[serde(default = "default_nats_cpu")]
    pub cpu: String,
    /// Per-broker memory, applied as request==limit.
    #[serde(default = "default_nats_memory")]
    pub memory: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl NatsSpec {
    /// True when the broker is operator-managed (no `externalUrl`).
    pub fn is_managed(&self) -> bool {
        self.external_url.is_none()
    }
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for NatsSpec {
    fn default() -> Self {
        Self {
            external_url: None,
            replicas: default_nats_replicas(),
            storage: default_nats_storage(),
            storage_class: None,
            cpu: default_nats_cpu(),
            memory: default_nats_memory(),
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
    /// Ready serving replicas (from the Deployment status).
    #[serde(default)]
    pub serving_ready_replicas: i32,
    /// Desired serving replicas (HPA floor at apply, or the live count).
    #[serde(default)]
    pub desired_replicas: i32,
    /// Effective shard count.
    #[serde(default)]
    pub shard_count: u32,
    /// Whether the NATS broker is Ready (managed) or assumed up (external).
    #[serde(default)]
    pub nats_ready: bool,
    /// Last human-readable reconcile message.
    #[serde(default)]
    pub message: String,
}

fn default_shard_count() -> u32 {
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
fn default_nats_replicas() -> i32 {
    1
}
fn default_nats_storage() -> String {
    "20Gi".into()
}
fn default_nats_cpu() -> String {
    "1".into()
}
fn default_nats_memory() -> String {
    "1Gi".into()
}
// CODEGEN-END
