// HANDWRITE-BEGIN gap="missing-generator:logic:defer-crd" tracker="#766" reason="Defer Kubernetes custom resource over shared cluster and backup contracts."
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "defer.dev",
    version = "v1alpha1",
    kind = "Defer",
    plural = "defers",
    shortname = "dfr",
    namespaced,
    status = "DeferStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct DeferSpec {
    #[serde(flatten)]
    pub cluster: service_k8s::ClusterSpec,
    #[serde(default = "default_storage")]
    pub storage: String,
    #[serde(default)]
    pub storage_class: Option<String>,
    #[serde(default = "default_grace_secs")]
    pub grace_secs: u64,
    #[serde(default)]
    pub log_level: Option<String>,
    #[serde(default = "default_auth")]
    pub auth: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_secret_provider_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_signing_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_signing_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_tls_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_seed_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<DeferBackupSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeferBackupSpec {
    #[serde(flatten)]
    pub policy: service_backup::ScheduledBackupPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token_secret: Option<String>,
}

impl std::ops::Deref for DeferBackupSpec {
    type Target = service_backup::ScheduledBackupPolicy;
    fn deref(&self) -> &Self::Target {
        &self.policy
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeferStatus {
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub observed_generation: i64,
    #[serde(default)]
    pub ready_replicas: i32,
    #[serde(default)]
    pub desired_replicas: i32,
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
