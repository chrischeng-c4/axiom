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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub struct ReshardWorkflowSpec {
    #[serde(default)]
    pub phase: ReshardPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_shard_count: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
pub enum ReshardPhase {
    #[default]
    Complete,
    PrepareSplit,
    Splitting,
    CatchingUp,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
    /// Bearer-token required; the token registry file comes from
    /// `tokensSecret` or `tokensSecretProviderClass`.
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
    /// Optional empty-PVC bootstrap seed. When set, serving pods restore this
    /// snapshot before WAL/raft catch-up. Supported seed URIs are exact
    /// `file://` SnapshotV1 JSON paths and, in backup-enabled builds, exact
    /// `s3://bucket/key` SnapshotV1 objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<ServingBootstrapSpec>,
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
            bootstrap: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_conditions: Vec<String>,
    #[serde(default)]
    pub message: String,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
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
            blocking_conditions,
            message,
        }
    }

    /// Live-usage-aware reshard status (#1319 R1): layers [`Self::reshard_status`]
    /// with real per-shard byte measurements instead of only formatting the
    /// configured percentages into a message. `shard_usage_bytes` maps
    /// `shard_index -> observed bytes` (see [`super::reconcile`]'s
    /// pod-`/metrics` measurement loop, the function's only caller).
    ///
    /// Reports whether the busiest shard has crossed `prepareAtPercent` /
    /// `urgentAtPercent` of `maxShardBytes`. It does **not** drive
    /// `workflow.phase` or move any data — the autonomous split executor
    /// (#1319 R2: computing a target topology, invoking
    /// [`crate::reshard::bucket_moves`] / [`crate::reshard::
    /// snapshot_reshard_batches`], and updating `shardMap.assignments`) is a
    /// separate, not-yet-implemented follow-up; a crossed threshold is
    /// reported here, never acted on.
    /// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-crd-rs.md#source
    pub fn reshard_status_with_usage(
        &self,
        shard_usage_bytes: &BTreeMap<u32, u64>,
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
