// <HANDWRITE gap="standardize:claim-code" tracker="projects-preview-src-model-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewEnvironment {
    pub api_version: String,
    pub kind: String,
    pub metadata: PreviewMetadata,
    pub spec: PreviewSpec,
    pub status: PreviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMetadata {
    pub name: String,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSpec {
    pub mr: u32,
    pub sha: String,
    pub image: String,
    pub app: String,
    pub namespace: String,
    pub base: BaseSpec,
    pub owner: String,
    pub ttl_hours: u32,
    pub route: RouteSpec,
    pub gke: GkeSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseSpec {
    pub namespace: String,
    pub workload: String,
    pub service: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSpec {
    pub host: String,
    pub target: String,
    pub cookie: String,
    pub header: String,
    pub service: String,
    pub service_port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GkeSpec {
    pub control_namespace: String,
    pub workload_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewStatus {
    pub phase: PreviewPhase,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PreviewPhase {
    Pending,
    Provisioning,
    Ready,
    Failed,
    Draining,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPlan {
    pub mr: u32,
    pub namespace: String,
    pub route_target: String,
    pub protected_namespaces: Vec<String>,
    pub action: CleanupAction,
    pub reason: String,
    pub delete_namespace: bool,
    pub delete_route_binding: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupAction {
    Keep,
    Drain,
    Delete,
}

// </HANDWRITE>
