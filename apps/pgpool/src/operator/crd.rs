// SPEC-MANAGED: apps/pgpool/tech-design/semantic/pgpool-crd-operator-control-plane.md#logic
// <HANDWRITE gap="missing-generator:logic:2b5d164e" tracker="#1575" reason="Define the namespaced Pgpool custom resource, provider/role endpoint budgets, and readiness plus connection-budget status schema.">
use std::collections::BTreeMap;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::k8s::{ControlPlaneStatus, PodControlPhase};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PgpoolEndpointProvider {
    #[default]
    PlainPostgres,
    CloudSql,
    AlloyDb,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PgpoolEndpointRole {
    #[default]
    Primary,
    ReadPool,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolEndpointBudgetSpec {
    pub name: String,
    #[serde(default)]
    pub provider: PgpoolEndpointProvider,
    #[serde(default)]
    pub role: PgpoolEndpointRole,
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    #[serde(default)]
    pub reserve: u32,
    #[serde(default)]
    pub safety_headroom: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configured_ceiling: Option<u32>,
    #[serde(default = "default_per_pod_quota")]
    pub per_pod_quota: u32,
}

impl Default for PgpoolEndpointBudgetSpec {
    fn default() -> Self {
        Self {
            name: "primary".into(),
            provider: PgpoolEndpointProvider::PlainPostgres,
            role: PgpoolEndpointRole::Primary,
            host: "postgres.default.svc".into(),
            port: default_postgres_port(),
            reserve: 10,
            safety_headroom: 10,
            configured_ceiling: None,
            per_pod_quota: default_per_pod_quota(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolResources {
    #[serde(default)]
    pub cpu: String,
    #[serde(default)]
    pub memory: String,
}

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "pgpool.axiom.dev",
    version = "v1alpha1",
    kind = "Pgpool",
    plural = "pgpools",
    shortname = "pgp",
    namespaced,
    status = "PgpoolStatus",
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#,
    printcolumn = r#"{"name":"Ready","type":"integer","jsonPath":".status.readyReplicas"}"#,
    printcolumn = r#"{"name":"Age","type":"date","jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolSpec {
    pub image: String,
    #[serde(default = "one")]
    pub replicas: u32,
    #[serde(default = "default_primary_endpoint")]
    pub primary_endpoint: String,
    #[serde(default = "default_endpoints")]
    pub endpoints: Vec<PgpoolEndpointBudgetSpec>,
    #[serde(default)]
    pub resources: PgpoolResources,
    #[serde(default = "default_grace_seconds")]
    pub termination_grace_period_seconds: u64,
}

impl PgpoolSpec {
    pub fn primary(&self) -> &PgpoolEndpointBudgetSpec {
        self.endpoints
            .iter()
            .find(|endpoint| endpoint.name == self.primary_endpoint)
            .or_else(|| self.endpoints.first())
            .unwrap_or_else(|| default_endpoint_ref())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolEndpointBudgetStatus {
    pub endpoint: String,
    pub provider: PgpoolEndpointProvider,
    pub role: PgpoolEndpointRole,
    pub effective_limit: u32,
    pub reserve: u32,
    pub non_pgpool_usage: u32,
    pub safety_headroom: u32,
    pub usable: u32,
    pub allocated: u32,
    pub available: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_scale_reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolPodBudgetStatus {
    pub pod: String,
    pub endpoint: String,
    pub quota: u32,
    pub phase: String,
    pub ready: bool,
    pub drain_requested: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain_deadline_epoch_seconds: Option<u64>,
    pub backend_active: u32,
    pub backend_idle: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PgpoolStatus {
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub observed_generation: i64,
    #[serde(default)]
    pub ready_replicas: i32,
    #[serde(default)]
    pub desired_replicas: i32,
    #[serde(default)]
    pub endpoints: Vec<PgpoolEndpointBudgetStatus>,
    #[serde(default)]
    pub pods: Vec<PgpoolPodBudgetStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_scale_reason: Option<String>,
    #[serde(default)]
    pub message: String,
}

impl PgpoolStatus {
    pub fn from_control_plane(
        spec: &PgpoolSpec,
        observed_generation: i64,
        ready_replicas: i32,
        control: &ControlPlaneStatus,
    ) -> Self {
        let endpoint_kinds: BTreeMap<_, _> = spec
            .endpoints
            .iter()
            .map(|item| (item.name.as_str(), (item.provider, item.role)))
            .collect();
        let endpoints = control
            .endpoints
            .iter()
            .map(|item| {
                let (provider, role) = endpoint_kinds
                    .get(item.endpoint.as_str())
                    .copied()
                    .unwrap_or_default();
                PgpoolEndpointBudgetStatus {
                    endpoint: item.endpoint.clone(),
                    provider,
                    role,
                    effective_limit: item.effective_limit,
                    reserve: item.reserve,
                    non_pgpool_usage: item.non_pgpool_usage,
                    safety_headroom: item.safety_headroom,
                    usable: item.usable,
                    allocated: item.allocated,
                    available: item.available,
                    blocked_scale_reason: item.blocked_scale_reason.clone(),
                }
            })
            .collect();
        let pods = control
            .pods
            .iter()
            .map(|item| PgpoolPodBudgetStatus {
                pod: item.pod.clone(),
                endpoint: item.endpoint.clone(),
                quota: item.quota,
                phase: pod_phase(item.phase).into(),
                ready: item.ready,
                drain_requested: item.drain_requested,
                drain_deadline_epoch_seconds: item.drain_deadline_epoch_seconds,
                backend_active: item.backend_active,
                backend_idle: item.backend_idle,
            })
            .collect();
        let desired_replicas = spec.replicas as i32;
        let phase = if control.blocked_scale_reason.is_some() {
            "Blocked"
        } else if desired_replicas > 0 && ready_replicas >= desired_replicas {
            "Ready"
        } else if ready_replicas > 0 {
            "Reconciling"
        } else {
            "Pending"
        };
        Self {
            phase: phase.into(),
            observed_generation,
            ready_replicas,
            desired_replicas,
            endpoints,
            pods,
            blocked_scale_reason: control.blocked_scale_reason.clone(),
            message: format!("{ready_replicas}/{desired_replicas} pgpool pods ready"),
        }
    }
}

fn pod_phase(phase: PodControlPhase) -> &'static str {
    match phase {
        PodControlPhase::Pending => "pending",
        PodControlPhase::Ready => "ready",
        PodControlPhase::Draining => "draining",
        PodControlPhase::Released => "released",
    }
}

fn one() -> u32 {
    1
}
fn default_postgres_port() -> u16 {
    5432
}
fn default_per_pod_quota() -> u32 {
    32
}
fn default_primary_endpoint() -> String {
    "primary".into()
}
fn default_endpoints() -> Vec<PgpoolEndpointBudgetSpec> {
    vec![PgpoolEndpointBudgetSpec::default()]
}
fn default_grace_seconds() -> u64 {
    60
}
fn default_endpoint_ref() -> &'static PgpoolEndpointBudgetSpec {
    static ENDPOINT: std::sync::OnceLock<PgpoolEndpointBudgetSpec> = std::sync::OnceLock::new();
    ENDPOINT.get_or_init(PgpoolEndpointBudgetSpec::default)
}
// </HANDWRITE>
