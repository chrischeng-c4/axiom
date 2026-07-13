---
id: projects-lumen-src-operator-crd-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/operator/crd.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/operator/crd.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `AuthMode` | apps/lumen/src/operator/crd.rs | enum | pub | 320 |  |
| `Autoscaling` | apps/lumen/src/operator/crd.rs | struct | pub | 461 |  |
| `LogFormat` | apps/lumen/src/operator/crd.rs | enum | pub | 297 |  |
| `LumenReshardStatus` | apps/lumen/src/operator/crd.rs | struct | pub | 513 | adds `convergence_remediation_restart_count: u32` (#1485 R1); adds `convergence_remediation_restarted_at: Option<u64>` (#1485 R1) |
| `LumenSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 39 |  |
| `LumenStatus` | apps/lumen/src/operator/crd.rs | struct | pub | 485 |  |
| `ReshardPhase` | apps/lumen/src/operator/crd.rs | enum | pub | 264 |  |
| `ReshardPolicy` | apps/lumen/src/operator/crd.rs | struct | pub | 163 |  |
| `ReshardWorkflowSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 198 | adds `converged_shard_map_version: Option<u64>` (#1458 R1); adds `last_cutover_shard_map_version: Option<u64>` (#1467 R7); adds `convergence_wait_started_at: Option<u64>` (#1485 R2); adds `convergence_remediation_restart_count: u32` (#1485 R1); adds `convergence_remediation_restarted_at: Option<u64>` (#1485 R1) |
| `ServingBackupSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 438 |  |
| `ServingBootstrapSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 415 |  |
| `ServingSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 347 |  |
| `ShardMapSpec` | apps/lumen/src/operator/crd.rs | struct | pub | 135 |  |
| `as_env` | apps/lumen/src/operator/crd.rs | function | pub | 308 | as_env(self) -> &'static str |
| `as_env` | apps/lumen/src/operator/crd.rs | function | pub | 335 | as_env(self) -> &'static str |
| `as_str` | apps/lumen/src/operator/crd.rs | function | pub | 274 | as_str(self) -> &'static str |
| `progress_percent` | apps/lumen/src/operator/crd.rs | function | pub | 283 | progress_percent(self) -> u8 |
| `reshard_status` | apps/lumen/src/operator/crd.rs | function | pub | 590 | reshard_status(&self) -> LumenReshardStatus |
| `reshard_status_with_usage` | apps/lumen/src/operator/crd.rs | function | pub | 675 | reshard_status_with_usage(&self, shard_usage_bytes: &BTreeMap<u32, u64>, measured_at_map_version: u64) -> LumenReshardStatus |
| `storage_pod_count` | apps/lumen/src/operator/crd.rs | function | pub | 571 | storage_pod_count(&self) -> i32 |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#rust-source-unit
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

use std::collections::BTreeMap;

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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct LumenSpec {
    /// Serving container image, e.g. `lumen:latest`. Required.
    pub image: String,

    /// Image pull policy. Defaults to `IfNotPresent`.
    #[serde(default)]
    pub image_pull_policy: Option<String>,

    /// Physical storage shard count. Data ownership is resolved through the
    /// versioned virtual-bucket map, not permanent `hash % shardCount`
    /// routing. HPA never changes this value.
    #[serde(default = "default_shard_count")]
    pub shard_count: u32,

    /// Versioned virtual-bucket map metadata. The default one-shard map keeps
    /// existing installs compatible; future reshard workflows bump `version`
    /// and move selected virtual buckets to new physical shards.
    #[serde(default)]
    pub shard_map: ShardMapSpec,

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
    /// `tokensSecret` or `tokensSecretProviderClass`).
    #[serde(default)]
    pub auth: AuthMode,

    /// Name of a Secret whose `token-registry.json` key is mounted at
    /// `/var/run/secrets/lumen/token-registry.json` and exposed to the serving
    /// process as `LUMEN_TOKEN_REGISTRY_FILE` when `auth: required`.
    /// `token-registry.json` is a JSON object of
    /// `{ "<token>": { "subject": "...", "roles": { "<collection_id>|*": "read|write|admin" } } }`.
    /// Ignored when `auth: off`. See also `tokensSecretProviderClass` for a
    /// Secret-free alternative; if both are set, this field wins.
    #[serde(default)]
    pub tokens_secret: Option<String>,

    /// Name of an existing `SecretProviderClass` (same namespace as this
    /// object) mounted via the Secrets Store CSI driver
    /// (`secrets-store.csi.k8s.io`) at the same path as `tokensSecret`
    /// (`/var/run/secrets/lumen/token-registry.json`, env
    /// `LUMEN_TOKEN_REGISTRY_FILE`), so the token registry's content never
    /// materializes as a k8s API object (`Secret` or `ConfigMap`) at all. The
    /// referenced `SecretProviderClass` must project a file named
    /// `token-registry.json` (same schema as `tokensSecret`'s Secret key).
    /// Ignored when `auth: off`. Mutual exclusion with `tokensSecret` is by
    /// precedence, not schema enforcement: if `tokensSecret` is also set, it
    /// wins (backward compatible) and this field is ignored. Rotation
    /// caveat: a CSI-mounted file only refreshes on the underlying value's
    /// rotation if the cluster's CSI driver has secret rotation enabled
    /// (e.g. GKE's managed add-on defaults it off); either way, lumen reads
    /// the registry once at serve startup, so picking up a rotated value
    /// requires a rolling restart regardless of the CSI driver's rotation
    /// setting.
    #[serde(default)]
    pub tokens_secret_provider_class: Option<String>,

    /// Stateless serving-fleet shape.
    #[serde(default)]
    pub serving: ServingSpec,

    /// Operator-owned storage reshard policy. HPA never changes storage
    /// ownership; this policy only prepares/recommends explicit shard topology
    /// changes.
    #[serde(default)]
    pub reshard_policy: ReshardPolicy,

    /// Emit a ServiceMonitor + PrometheusRule. Requires the prometheus-operator
    /// CRDs (`monitoring.coreos.com/v1`) to be installed in the cluster.
    #[serde(default)]
    pub observability: bool,
}

/// Versioned virtual-bucket map control-plane metadata.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ShardMapSpec {
    #[serde(default)]
    pub version: u64,
    #[serde(default = "default_virtual_bucket_count")]
    pub virtual_bucket_count: u32,
    /// Optional explicit `bucket -> physical shard` assignments. Empty means
    /// derive the deterministic balanced assignment `bucket % shardCount`;
    /// reshard workflows set this to move selected buckets without changing
    /// every key.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assignments: Vec<u32>,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for ShardMapSpec {
    fn default() -> Self {
        Self {
            version: 0,
            virtual_bucket_count: default_virtual_bucket_count(),
            assignments: Vec::new(),
        }
    }
}

/// Storage-pressure policy for rare, operator-owned shard split workflows.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ReshardPolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shard_bytes: Option<u64>,
    #[serde(default = "default_reshard_prepare_percent")]
    pub prepare_at_percent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_at_percent: Option<u8>,
    #[serde(default = "default_reshard_urgent_percent")]
    pub urgent_at_percent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_shards: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_bytes_per_sec: Option<u64>,
    #[serde(default)]
    pub workflow: ReshardWorkflowSpec,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl Default for ReshardPolicy {
    fn default() -> Self {
        Self {
            max_shard_bytes: None,
            prepare_at_percent: default_reshard_prepare_percent(),
            start_at_percent: None,
            urgent_at_percent: default_reshard_urgent_percent(),
            max_shards: None,
            migration_bytes_per_sec: None,
            workflow: ReshardWorkflowSpec::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ReshardWorkflowSpec {
    #[serde(default)]
    pub phase: ReshardPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_shard_count: Option<u32>,
    /// `shardMap.version` the reshard driver has confirmed every serving
    /// pod is `Ready` on (#1458 R1) — the persisted checkpoint
    /// `reshard_driver::advance_convergence` compares `spec.shardMap.
    /// version` against to decide whether the post-cutover write-pause
    /// fence must stay armed. `None` (or a value behind the current
    /// `shardMap.version`) means convergence for the current map is still
    /// pending or was never confirmed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub converged_shard_map_version: Option<u64>,
    /// `shardMap.version` the reshard driver's own cutover last patched
    /// into `spec.shardMap.version` (#1467 R7), stamped in the exact same
    /// `advance_catching_up_fenced` patch call that sets `shardMap.
    /// version`/`phase: Complete`. The ONLY writer of this field — a
    /// hand-authored or backup-restored `spec.shardMap` never sets it, so
    /// it stays behind (usually `None`) forever for such a CR.
    /// `advance_convergence` requires this to equal the current
    /// `shardMap.version` before engaging the post-cutover write-pause
    /// fence loop at all, closing the gap where convergence would
    /// otherwise fence indefinitely over a topology the driver never
    /// actually changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_cutover_shard_map_version: Option<u64>,
    /// Epoch-seconds wall-clock timestamp `reshard_driver::advance_convergence`
    /// first observed the *current* `shardMap.version`'s post-cutover
    /// convergence wait as pending (#1485 R2) — stamped once, on the first
    /// `AwaitingTopologyConvergence` tick, in the same `Patch::Merge` style
    /// `lastCutoverShardMapVersion` already uses (spec-is-checkpoint, not
    /// driver memory). Cleared (patched to `null`) the moment convergence is
    /// confirmed, so it is always either `None` or the start of the wait
    /// still in progress. `reshard_driver::convergence_stall_condition`
    /// computes the `topologyConvergenceStalled` budget directly from this
    /// field, so both the budget and the raised condition survive an
    /// operator restart — replacing the prior process-local-cache-only
    /// computation, which reset to zero on every restart.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convergence_wait_started_at: Option<u64>,
    /// Count of bounded remediation rolling-restart re-triggers
    /// `reshard_driver::advance_convergence` has fired for the current
    /// convergence-stall episode — the same wait `convergenceWaitStartedAt`
    /// tracks (#1485 R1). Bounded to at most `1`: once the stall budget is
    /// exceeded with the ConfigMap-race signature (StatefulSet rollout
    /// complete but some pod still reporting the old shard-map version), the
    /// driver calls `ClusterControl::trigger_rolling_restart` exactly once
    /// per episode and bumps this to `1`; a later stall in the same episode
    /// never re-triggers. Reset to `0` alongside `convergenceWaitStartedAt`
    /// once the episode resolves.
    #[serde(default)]
    pub convergence_remediation_restart_count: u32,
    /// Epoch-seconds timestamp of the last remediation rolling-restart
    /// re-trigger this episode, if any (#1485 R1) — surfaced alongside
    /// `convergenceRemediationRestartCount` in `status.reshard` so operators
    /// can see when the self-heal fired without reading driver logs. `None`
    /// until `convergenceRemediationRestartCount` first becomes non-zero;
    /// cleared together with it once the episode resolves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convergence_remediation_restarted_at: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum ReshardPhase {
    #[default]
    Complete,
    PrepareSplit,
    Splitting,
    CatchingUp,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl ReshardPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "Complete",
            Self::PrepareSplit => "PrepareSplit",
            Self::Splitting => "Splitting",
            Self::CatchingUp => "CatchingUp",
        }
    }

    pub fn progress_percent(self) -> u8 {
        match self {
            Self::Complete => 100,
            Self::PrepareSplit => 10,
            Self::Splitting => 60,
            Self::CatchingUp => 90,
        }
    }
}

/// Log output format.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum LogFormat {
    /// Structured one-line-per-event JSON (prod/staging).
    Json,
    /// Human-readable multi-line (dev).
    #[default]
    Pretty,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum AuthMode {
    /// Open API (dev / trusted network). Serialized as `disabled` — NOT `off`,
    /// which YAML 1.1 (kubectl / go-yaml) would parse as the boolean `false`
    /// and corrupt the CRD enum/default.
    #[default]
    #[serde(rename = "disabled")]
    Off,
    /// Bearer-token required; the token registry file comes from
    /// `tokensSecret` or `tokensSecretProviderClass`.
    Required,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
    /// Optional empty-PVC bootstrap seed. When set, serving pods restore this
    /// snapshot before WAL/raft catch-up. Supported seed URIs are exact
    /// `file://` SnapshotV1 JSON paths and, in backup-enabled builds, exact
    /// `s3://bucket/key` SnapshotV1 objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<ServingBootstrapSpec>,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
            bootstrap: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ServingBootstrapSpec {
    /// SnapshotV1 JSON seed URI. Use an exact `file://` path or
    /// `s3://bucket/key` object, not a backup prefix.
    pub seed_uri: String,
    /// Optional read throttle advertised to operators/status. The current
    /// source primitive reads one object per bootstrap; transfer shaping can be
    /// enforced by the object-store client/proxy or a future streaming reader.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes_per_sec: Option<u64>,
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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct Autoscaling {
    /// Floor (also the StatefulSet's apply-time replica count in HPA mode).
    pub min_replicas: i32,
    /// Ceiling.
    pub max_replicas: i32,
    /// Target average CPU utilization (%) — read QPS proxied by CPU.
    pub target_cpu_utilization: i32,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
    /// Reshard workflow status. Present even before a split starts so agents
    /// can distinguish "complete" from "unknown policy".
    #[serde(default)]
    pub reshard: LumenReshardStatus,
    /// Last human-readable reconcile message.
    #[serde(default)]
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct LumenReshardStatus {
    #[serde(default)]
    pub phase: String,
    // Schema default corrected (#1319 R3) to match the actual runtime value
    // (`ReshardPolicy::default().max_shard_bytes.is_none() == true`), not
    // `bool::default()` (`false`) — the CRD's declared default used to
    // disagree with what the operator always reports at the CRD's own
    // `reshardPolicy` defaults.
    #[serde(default)]
    #[schemars(default = "default_reshard_recommendation_only")]
    pub recommendation_only: bool,
    #[serde(default)]
    pub progress_percent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_shard_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_bytes_per_sec: Option<u64>,
    /// Highest observed percent of `maxShardBytes` across shards, from the
    /// live per-shard usage measurement (#1319 R1;
    /// [`super::reconcile`]'s pod-`/metrics` measurement loop is the only
    /// caller of [`LumenSpec::reshard_status_with_usage`], which sets this).
    /// `None` when `maxShardBytes` is unset or usage has not been measured
    /// yet — the plain [`LumenSpec::reshard_status`] never sets it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_observed_percent: Option<u8>,
    /// The `spec.shardMap.version` live when [`Self::max_observed_percent`]
    /// was captured (#1386 R1/R2) — the usage measurement's freshness
    /// generation. [`LumenSpec::reshard_status_with_usage`] only reports a
    /// crossed threshold when this equals the CR's *current*
    /// `spec.shardMap.version`; a mismatch means the measurement predates
    /// the most recent split's cutover (immediately after `Complete`, the
    /// shard-usage cache almost always still holds exactly this — the live
    /// #1384 proof bug this field closes) and the status instead reports
    /// `"usageStalePostCutover"`, holding until a fresh post-cutover
    /// scrape lands. `None` alongside `max_observed_percent == None` (no
    /// measurement yet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_measured_at_map_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_conditions: Vec<String>,
    #[serde(default)]
    pub message: String,
    /// Mirrors `spec.reshardPolicy.workflow.convergenceRemediationRestartCount`
    /// (#1485 R1) — count of bounded remediation rolling-restart re-triggers
    /// the reshard driver has fired for the current convergence-stall
    /// episode, so operators can see the self-heal fired without reading
    /// `spec`. `status_patch` copies this straight from the spec field.
    #[serde(default)]
    pub convergence_remediation_restart_count: u32,
    /// Mirrors `spec.reshardPolicy.workflow.convergenceRemediationRestartedAt`
    /// (#1485 R1) — epoch-seconds timestamp of the last remediation restart
    /// re-trigger, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convergence_remediation_restarted_at: Option<u64>,
}

/// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
impl LumenSpec {
    pub fn storage_pod_count(&self) -> i32 {
        if self.replicas_per_shard > 1 {
            (self.shard_count * self.replicas_per_shard) as i32
        } else if self.shard_count > 1 {
            self.shard_count as i32
        } else {
            // Single shard, single member, no raft consensus (#1317): every
            // pod's `shard_index` (`ordinal % shard_count`) collapses to 0,
            // so more than one live pod here means multiple uncoordinated
            // local copies behind one Service — confirmed empirically on a
            // kind cluster (a write via one pod is invisible on the others;
            // a load-balanced Service returns divergent results for the
            // same read). Clamp to exactly 1 regardless of the CR's
            // `serving.autoscaling` bounds; CPU-driven scaling requires
            // opting into `replicasPerShard > 1` (raft-HA).
            1
        }
    }

    pub fn reshard_status(&self) -> LumenReshardStatus {
        let policy = &self.reshard_policy;
        let recommendation_only = policy.max_shard_bytes.is_none();
        let mut blocking_conditions = Vec::new();
        if recommendation_only {
            blocking_conditions.push("maxShardBytesUnset".to_string());
        }
        if policy.max_shards.is_some_and(|max| self.shard_count >= max) {
            blocking_conditions.push("maxShardsReached".to_string());
        }
        let target = policy.workflow.target_shard_count.or_else(|| {
            policy.max_shards.map(|max| {
                if self.shard_count < max {
                    self.shard_count + 1
                } else {
                    self.shard_count
                }
            })
        });
        let message = if recommendation_only {
            "maxShardBytes unset; operator reports recommendations only and will not auto-split"
                .to_string()
        } else if policy.max_shards.is_some_and(|max| self.shard_count >= max) {
            "maxShards reached; reshard workflow requires an explicit higher limit".to_string()
        } else {
            format!(
                "prepare at {}%, start at {}%, urgent at {}%",
                policy.prepare_at_percent,
                policy.start_at_percent.unwrap_or(policy.prepare_at_percent),
                policy.urgent_at_percent
            )
        };
        LumenReshardStatus {
            phase: policy.workflow.phase.as_str().to_string(),
            recommendation_only,
            progress_percent: policy.workflow.phase.progress_percent(),
            target_shard_count: target,
            migration_bytes_per_sec: policy.migration_bytes_per_sec,
            max_observed_percent: None,
            usage_measured_at_map_version: None,
            blocking_conditions,
            message,
            convergence_remediation_restart_count: policy
                .workflow
                .convergence_remediation_restart_count,
            convergence_remediation_restarted_at: policy
                .workflow
                .convergence_remediation_restarted_at,
        }
    }

    /// Live-usage-aware reshard status (#1319 R1): layers [`Self::reshard_status`]
    /// with real per-shard byte measurements instead of only formatting the
    /// configured percentages into a message. `shard_usage_bytes` maps
    /// `shard_index -> observed bytes`; `measured_at_map_version` is the
    /// `spec.shardMap.version` that was live on this CR when that usage was
    /// scraped (see [`super::reconcile`]'s pod-`/metrics` measurement loop,
    /// the function's only caller).
    ///
    /// Reports whether the busiest shard has crossed `prepareAtPercent` /
    /// `urgentAtPercent` of `maxShardBytes` — but only once
    /// `measured_at_map_version` matches this CR's *current*
    /// `shard_map.version` (#1386 R1/R2). A split's cutover bumps
    /// `shard_map.version` in the very same patch that follows evicting
    /// moved documents from their old shard, so a mismatch means the
    /// measurement predates that cutover and still reflects pre-eviction
    /// usage — most visibly, immediately after a split reaches `Complete`,
    /// when the shard-usage cache has not yet re-scraped (the exact live
    /// #1384 bug: a stale post-migration reading re-crossed the threshold
    /// and cascaded straight into an unwarranted second split). While
    /// stale, this reports `"usageStalePostCutover"` instead of a
    /// threshold-crossed condition, holding until a fresh post-cutover
    /// scrape lands; a genuinely still-hot shard can still trigger the next
    /// split, but only once the measurement itself is proven post-cutover.
    ///
    /// This function itself does **not** drive `workflow.phase` or move any
    /// data — it only computes the status this tick. The autonomous split
    /// executor (#1319 R2, #1381: computing a target topology, invoking
    /// [`crate::reshard::bucket_moves`] / [`crate::reshard::
    /// snapshot_reshard_batches`], and updating `shardMap.assignments`) is a
    /// separate loop ([`crate::operator::reshard_driver::
    /// should_start_split`] / `drive_tick`) that reads the
    /// `blockingConditions` this function writes and acts on them
    /// independently.
    /// @spec apps/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
    pub fn reshard_status_with_usage(
        &self,
        shard_usage_bytes: &BTreeMap<u32, u64>,
        measured_at_map_version: u64,
    ) -> LumenReshardStatus {
        let mut status = self.reshard_status();
        let Some(max_shard_bytes) = self.reshard_policy.max_shard_bytes else {
            // recommendation-only: nothing to compare usage against.
            return status;
        };
        if max_shard_bytes == 0 {
            return status;
        }
        let Some((&busiest_shard, &busiest_bytes)) =
            shard_usage_bytes.iter().max_by_key(|(_, bytes)| **bytes)
        else {
            // Usage not measured yet this tick; keep the policy-only status.
            return status;
        };

        let percent = ((busiest_bytes as f64 / max_shard_bytes as f64) * 100.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        status.max_observed_percent = Some(percent);
        status.usage_measured_at_map_version = Some(measured_at_map_version);

        if measured_at_map_version != self.shard_map.version {
            // #1386 R1: this measurement predates the most recent cutover
            // (or, less likely, raced ahead of a status write that hasn't
            // observed it yet) — never let a stale reading drive
            // `should_start_split`, no matter how urgent the stale
            // percentage looks.
            status
                .blocking_conditions
                .push("usageStalePostCutover".to_string());
            status.message = format!(
                "usage measured at shardMap version {measured_at_map_version}, but the CR is \
                 now at version {}; holding for a fresh post-cutover measurement before \
                 evaluating the next split",
                self.shard_map.version
            );
            return status;
        }

        let policy = &self.reshard_policy;
        let prepare_at = policy.start_at_percent.unwrap_or(policy.prepare_at_percent);
        status.message = if percent >= policy.urgent_at_percent {
            status
                .blocking_conditions
                .push("urgentThresholdCrossed".to_string());
            format!(
                "urgent threshold crossed: shard {busiest_shard} at {percent}% of maxShardBytes \
                 (urgent {}%)",
                policy.urgent_at_percent
            )
        } else if percent >= prepare_at {
            status
                .blocking_conditions
                .push("prepareThresholdCrossed".to_string());
            format!(
                "prepare threshold crossed: shard {busiest_shard} at {percent}% of \
                 maxShardBytes (prepare {prepare_at}%)"
            )
        } else {
            format!(
                "shard {busiest_shard} at {percent}% of maxShardBytes; below prepare \
                 threshold ({prepare_at}%)"
            )
        };
        status
    }
}

fn default_shard_count() -> u32 {
    1
}
fn default_virtual_bucket_count() -> u32 {
    crate::routing::DEFAULT_VIRTUAL_BUCKET_COUNT
}
fn default_replicas_per_shard() -> u32 {
    1
}
fn default_reshard_prepare_percent() -> u8 {
    50
}
fn default_reshard_urgent_percent() -> u8 {
    85
}
// #1319 R3: the declared CRD schema default must match the actual runtime
// default (`ReshardPolicy::default().max_shard_bytes.is_none() == true`),
// not `bool::default()` (`false`).
fn default_reshard_recommendation_only() -> bool {
    true
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `apps/lumen/src/operator/crd.rs` captured during lumen
      standardization onto the per-file codegen ladder.
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1381: doc-comment-only update on `reshard_status_with_usage`
      pointing at the new `reshard_driver` module (`should_start_split` /
      `drive_tick`) that now consumes `blockingConditions` and actually
      drives `workflow.phase`.
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1386 R1/R2/R3: added `LumenReshardStatus.usage_measured_at_map_version`
      and a new `measured_at_map_version: u64` parameter on
      `reshard_status_with_usage`, so a stale (pre-cutover) usage
      measurement — the exact shape the shard-usage cache holds for one
      scrape tick right after a split completes — reports
      `"usageStalePostCutover"` instead of a crossed threshold, closing the
      #1384 cascade-split bug without any change to
      `reshard_driver::should_start_split`.
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1458 R1: added `ReshardWorkflowSpec.converged_shard_map_version:
      Option<u64>` — the persisted checkpoint
      `reshard_driver::advance_convergence` compares against
      `spec.shardMap.version` to decide whether the post-cutover
      write-pause fence must stay armed, resumable across a driver
      restart since it derives purely from this field, not driver memory.
  - path: apps/lumen/src/operator/crd.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1467 R7: added `ReshardWorkflowSpec.last_cutover_shard_map_version:
      Option<u64>` — stamped by the reshard driver's own cutover in the
      exact same `advance_catching_up_fenced` patch call that sets
      `shardMap.version` / `phase: Complete`. `advance_convergence` now
      requires this field to equal the current `shardMap.version` before
      engaging the post-cutover write-pause fence loop at all, so a
      manually-authored or backup-restored `shardMap` (which never sets
      this field) no longer causes convergence to fence indefinitely over
      a topology the driver never actually changed.
```
