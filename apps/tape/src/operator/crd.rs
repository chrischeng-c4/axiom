// HANDWRITE-BEGIN gap="missing-generator:logic:bfdc7475" tracker="pending-tracker" reason="TapeSpec CustomResource (group tape.dev, v1alpha1, kind Tape, plural tapes, shortname tp, namespaced, status TapeStatus, printcolumns Phase/Ready/Age): #[serde(flatten)] cluster: service_k8s::ClusterSpec (shardCount defaults 1, pinned by the render -- tape is a single raft group) + storage (default 10Gi) + storageClass + graceSecs (default 10) + logLevel (Option) + auth (closed AuthMode enum disabled|required, defaulting to required) + tokensSecret (Option<String>) + serviceAccountName (Option<String>). TapeStatus { phase, observedGeneration, readyReplicas, desiredReplicas, message, conditions }."
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

    /// Request-auth mode for the data plane: `required` (the default — supply
    /// a token registry via `tokensSecret` or `tokensSecretProviderClass`) or
    /// `disabled`.
    ///
    /// `required` is the default because the other way round, forgetting this
    /// field ships an open cluster and nothing says so; forgetting it now
    /// fails startup with a message naming the field to set. `disabled`
    /// remains a one-word opt-out for local development (#2765).
    ///
    /// Spelled `disabled`, not `off`: YAML 1.1 reads a bare `off` as the
    /// boolean `false`. (`off` is what the serving process's own `TAPE_AUTH`
    /// env var takes — the two spellings are not interchangeable.)
    #[serde(default)]
    pub auth: AuthMode,

    /// Name of the Secret carrying the bearer-token registry (key
    /// `token-registry.json`). When set with `auth: required`, the render
    /// mounts it read-only at `/var/run/secrets/tape` and injects
    /// `TAPE_TOKEN_REGISTRY_FILE` (relay/lumen's pattern). Tape watches a
    /// changed projection and atomically applies only a valid replacement
    /// without a pod restart. Ignored when `auth: disabled`. Setting **both**
    /// this and `tokensSecretProviderClass` is rejected by the CRD schema
    /// (#2765), because silently preferring one leaves an operator reading
    /// credentials that are not the ones being served.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret: Option<String>,

    /// Name of a Secrets Store CSI `SecretProviderClass` that projects the
    /// same `token-registry.json` file as [`Self::tokens_secret`]. When the
    /// CSI driver refreshes that file, Tape's watcher applies valid rotations
    /// without a pod restart. Ignored when `auth: disabled`. Mutual exclusion
    /// with `tokensSecret` is enforced by the CRD schema
    /// (`x-kubernetes-validations`), so setting both is rejected at
    /// `kubectl apply` rather than resolved by a precedence rule nothing
    /// surfaces (#2765).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret_provider_class: Option<String>,

    /// CSI driver name for the `tokensSecretProviderClass` projection.
    /// Defaults to the community `secrets-store.csi.k8s.io`; GKE's managed
    /// Secrets Store add-on registers `secrets-store-gke.csi.k8s.io`, so GKE
    /// instances must set that value (#2456).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret_csi_driver: Option<String>,

    /// Exact backup object URI consulted to seed a replica whose data
    /// directory is still EMPTY before Raft catch-up. Bootstrap-if-empty
    /// (#2468): the server probes the durable data directory first — a pod
    /// that already has raft state (including a routine restart onto its own
    /// PVC) skips the seed and boots from that existing state instead of
    /// refusing, so this field may harmlessly stay set on the CR after a
    /// successful bootstrap. It only ever seeds a fresh (empty) data
    /// directory, so this is cold recovery rather than live replica
    /// synchronization. Recommended hygiene: remove it once the bootstrap
    /// converges anyway, so the CR does not silently re-seed if the PVC is
    /// ever recreated (e.g. deleted-and-reprovisioned) — the field's presence
    /// alone would then look like a fresh empty PVC again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_seed_uri: Option<String>,

    /// Data-plane request body size limit (bytes). Requests with
    /// `Content-Length` exceeding this are rejected with 413; streamed bodies
    /// are bounded mid-read. Defaults to 8 MiB. Unset means the server
    /// default (#2484).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_limit_bytes: Option<u64>,

    /// Declarative topic/subscription provisioning: a list of topics and their
    /// pre-created subscriptions. The serve path (after the journal is ready)
    /// idempotently ensures each subscription, tolerating `AlreadyExists` errors.
    /// Topic creation is implicit (no journal mutation); declaring a topic alone
    /// is documentation + future-proofing for per-topic config (retention/quotas
    /// — #2550). **Additive-only**: removing an entry from this list never deletes
    /// anything (deletion lifecycle is #2549's decision). Cross-ref: #2557.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<TapeTopicSpec>>,

    /// Emit a `ServiceMonitor` + `PrometheusRule` alongside the workload
    /// (#2575), carrying the same four SLO alerts the hand-maintained
    /// `k8s/components/observability` kustomize component ships — including
    /// #2485's seed-failure and consumer-liveness runbooks.
    ///
    /// Opt-in, and default-off on purpose: both are `monitoring.coreos.com/v1`
    /// kinds, so a cluster without the Prometheus Operator CRDs installed
    /// would reject them. Leaving it off keeps a vanilla cluster installable;
    /// turning it on there is a reconcile error the operator reports, not a
    /// surprise the CR author meets at install time. Same shape and default as
    /// lumen's `spec.observability`.
    #[serde(default)]
    pub observability: bool,

    /// Optional scheduled backup. When set, the operator renders a CronJob
    /// that runs `tape backup` against this instance's client Service on the
    /// declared schedule (#2574). Unset renders no CronJob — the previous
    /// behavior, and still the default.
    ///
    /// The `tape backup` CLI verb and the `/admin/backup` endpoint it pulls
    /// from both already existed (#1329); what was missing was any
    /// declarative way to schedule them, so the only route was running the
    /// verb by hand or hand-authoring a CronJob outside the operator — which
    /// then did not track the CR's image, ServiceAccount, or auth wiring.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<TapeBackupSpec>,

    /// Optional external ServiceAccount name for the workload StatefulSet.
    /// When set, the operator skips rendering the `<instance>` ServiceAccount
    /// entirely and configures the workload StatefulSet to use this name
    /// instead (#2581). `<name>-backup` identity is unaffected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account_name: Option<String>,
}

/// Scheduled-backup projection: the shared
/// [`service_backup::ScheduledBackupPolicy`] (`schedule`, `destination`,
/// `retentionSecs`) plus tape's own admin-token wiring.
///
/// The policy is `#[serde(flatten)]`ed rather than nested so the CRD carries
/// the shared fields directly and every service operator that schedules a
/// backup keeps one schema for them — defer's `DeferBackupSpec` is the same
/// shape. A structural CRD schema cannot represent the runtime
/// `BackupDestination` enum, which is why the flat `destination` URI string
/// is the projected form.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TapeBackupSpec {
    /// `schedule` (cron expression), `destination` (`file://` / `s3://` /
    /// `gs://` URI), and optional `retentionSecs`.
    #[serde(flatten)]
    pub policy: service_backup::ScheduledBackupPolicy,

    /// Name of a Secret holding a bearer token with `admin` on `*`, projected
    /// into the CronJob as `TAPE_BACKUP_TOKEN` (key `token`). Required when
    /// the instance runs `auth: required`; omit for `auth: disabled`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token_secret: Option<String>,
}

impl std::ops::Deref for TapeBackupSpec {
    type Target = service_backup::ScheduledBackupPolicy;
    fn deref(&self) -> &Self::Target {
        &self.policy
    }
}

/// Topic declaration with optional subscriptions.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TapeTopicSpec {
    /// Topic name.
    pub name: String,
    /// Pre-created subscriptions for this topic (default: empty).
    #[serde(default)]
    pub subscriptions: Vec<String>,
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
    /// Kubernetes-convention convergence conditions (#3054): `Ready`,
    /// `Progressing`, `StorageHealthy`, `BackupConfigured`. This is the
    /// surface `kubectl wait --for=condition=Ready`, Argo CD health
    /// assessment, and Flux readiness gates read; `phase` is unchanged and
    /// still populated, so nothing already consuming it breaks.
    ///
    /// `lastTransitionTime` is stamped by the reconcile loop, not here — see
    /// [`super::reconcile`]'s no-I/O `status_patch` contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<service_k8s::Condition>,
}

fn default_storage() -> String {
    "10Gi".into()
}
fn default_grace_secs() -> u64 {
    10
}

/// Whether the data-plane API requires a bearer token.
///
/// A closed enum, not a flat string: as a `String` every value except the
/// exact literal `"required"` rendered an open data plane, so `auth: requried`
/// applied cleanly and the only signal was the *absence* of an env var in a
/// pod nobody inspects (#2765). The API server now rejects anything outside
/// this set, naming the field.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// Open API (dev / trusted network) — an explicit opt-out, never the
    /// default. Serialized as `disabled` — NOT `off`, which YAML 1.1
    /// (kubectl / go-yaml) would parse as the boolean `false` and corrupt the
    /// CRD enum/default.
    #[serde(rename = "disabled")]
    Off,
    /// Bearer token required; the registry file comes from `tokensSecret` or
    /// `tokensSecretProviderClass`. The default, so a `Tape` that omits
    /// `spec.auth` fails startup asking for credentials instead of serving an
    /// open API silently.
    #[default]
    Required,
}

impl AuthMode {
    /// The `TAPE_AUTH` value the serving binary expects.
    ///
    /// Deliberately *not* the CRD spelling: the wire keeps `off` (see
    /// `auth::AUTH_MODE_ENV` and `tape serve --auth`), the schema uses
    /// `disabled`. An env var is never parsed as YAML, so `off` is safe there
    /// and changing it would break every existing deployment's serving args.
    pub fn as_env(self) -> &'static str {
        match self {
            AuthMode::Off => "off",
            AuthMode::Required => "required",
        }
    }
}
// HANDWRITE-END
