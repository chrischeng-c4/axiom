//! Platform capacity catalog consumption and placement derivation for Lumen.
//!
//! Lumen instances consume precreated, shared, autoscaled GCE node pools
//! provisioned via Terraform without exposing internal node pool names or
//! invented service tiers.
//!
//! The platform capacity catalog is published as a Kubernetes ConfigMap
//! (`lumen-capacity-catalog` in `lumen-system` namespace) by Terraform
//! (`apps/lumen/terraform/modules/lumen-capacity/catalog.tf`), mapping direct GCE
//! machine types to `lumen.axiom.dev/capacity-profile` labels and tolerations.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

/// Default initial GCE machine type.
pub const DEFAULT_INITIAL_MACHINE_TYPE: &str = "e2-standard-2";

/// Default data volume storage request.
pub const DEFAULT_DATA_STORAGE: &str = "10Gi";

/// Default storage class for persistent data volumes.
pub const DEFAULT_STORAGE_CLASS: &str = "standard-rwo";

/// Default backing disk type.
pub const DEFAULT_DISK_TYPE: &str = "pd-balanced";

/// Default namespace for the in-cluster capacity catalog ConfigMap.
pub const DEFAULT_CATALOG_NAMESPACE: &str = "lumen-system";

/// Default name of the in-cluster capacity catalog ConfigMap.
pub const DEFAULT_CATALOG_CONFIG_MAP_NAME: &str = "lumen-capacity-catalog";

/// In-cluster capacity catalog published by the platform Terraform module.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityCatalog {
    #[serde(default = "default_catalog_version")]
    pub version: String,
    #[serde(default)]
    pub entries: Vec<CatalogEntry>,
}

fn default_catalog_version() -> String {
    "1.0.0".to_string()
}

impl CapacityCatalog {
    pub fn new(entries: Vec<CatalogEntry>) -> Self {
        Self {
            version: default_catalog_version(),
            entries,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

/// Fetch the capacity catalog from the in-cluster ConfigMap published by Terraform.
pub async fn fetch_capacity_catalog(
    client: &kube::Client,
    namespace: &str,
    name: &str,
) -> Result<CapacityCatalog, Rejection> {
    use k8s_openapi::api::core::v1::ConfigMap;
    let cm_api: kube::Api<ConfigMap> = kube::Api::namespaced(client.clone(), namespace);
    let cm = cm_api.get(name).await.map_err(|err| Rejection {
        reason: RejectionReason::CatalogMissing,
        field_path: "catalog".to_string(),
        message: format!("failed to read capacity catalog ConfigMap `{namespace}/{name}`: {err}"),
    })?;
    let data = cm.data.ok_or_else(|| Rejection {
        reason: RejectionReason::CatalogMissing,
        field_path: "catalog".to_string(),
        message: format!("ConfigMap `{namespace}/{name}` has no data"),
    })?;
    let raw_json = data.get("catalog.json").ok_or_else(|| Rejection {
        reason: RejectionReason::CatalogIncompatible,
        field_path: "catalog".to_string(),
        message: format!("ConfigMap `{namespace}/{name}` missing `catalog.json` key"),
    })?;
    CapacityCatalog::from_json(raw_json).map_err(|err| Rejection {
        reason: RejectionReason::CatalogIncompatible,
        field_path: "catalog".to_string(),
        message: format!("failed to parse `catalog.json` in `{namespace}/{name}`: {err}"),
    })
}

/// A single catalog entry representing an available direct GCE machine type.
/// Deserializes the exact 7 fields published by `catalog.tf`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogEntry {
    pub machine_type: String,
    pub selector: String,
    pub stable_selector: StableSelector,
    pub max_nodes: u32,
    pub min_nodes: u32,
    pub lifecycle_state: String,
    #[serde(default)]
    pub pool_group: Option<String>,
}

impl CatalogEntry {
    pub fn new(
        machine_type: &str,
        selector_key: &str,
        lifecycle_state: &str,
        max_nodes: u32,
    ) -> Self {
        Self {
            machine_type: machine_type.to_string(),
            selector: format!("{selector_key}={machine_type}"),
            stable_selector: StableSelector {
                key: selector_key.to_string(),
                value: machine_type.to_string(),
            },
            max_nodes,
            min_nodes: 0,
            lifecycle_state: lifecycle_state.to_string(),
            pool_group: Some("lumen-data".to_string()),
        }
    }
}

/// Key-value selector pair for a catalog entry.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StableSelector {
    pub key: String,
    pub value: String,
}

/// Reason code for capacity admission or preflight rejection.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    UnsupportedMachineType,
    CatalogMissing,
    CatalogAmbiguous,
    CatalogDraining,
    CapacityFull,
    CatalogIncompatible,
    InsufficientAllocatable,
    DataMemberNodeConflict,
    TransitionNotAllowed,
    MonetaryPolicyNotAllowed,
}

impl RejectionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedMachineType => "unsupported_machine_type",
            Self::CatalogMissing => "catalog_missing",
            Self::CatalogAmbiguous => "catalog_ambiguous",
            Self::CatalogDraining => "catalog_draining",
            Self::CapacityFull => "capacity_full",
            Self::CatalogIncompatible => "catalog_incompatible",
            Self::InsufficientAllocatable => "insufficient_allocatable",
            Self::DataMemberNodeConflict => "data_member_node_conflict",
            Self::TransitionNotAllowed => "transition_not_allowed",
            Self::MonetaryPolicyNotAllowed => "monetary_policy_not_allowed",
        }
    }
}

/// Structured rejection verdict from capacity validation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Rejection {
    pub reason: RejectionReason,
    pub field_path: String,
    pub message: String,
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.reason.as_str(), self.message, self.field_path)
    }
}

impl std::error::Error for Rejection {}

/// Create-time machine type specification for Lumen data plane.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacitySpec {
    pub initial_machine_type: String,
}

impl CapacitySpec {
    pub fn default() -> Self {
        Self {
            initial_machine_type: DEFAULT_INITIAL_MACHINE_TYPE.to_string(),
        }
    }
}

impl Default for CapacitySpec {
    fn default() -> Self {
        Self::default()
    }
}

/// Validate that a given machine type string is a valid direct GCE machine type.
pub fn is_valid_direct_gce_machine_type(mt: &str) -> bool {
    let parts: Vec<&str> = mt.split('-').collect();
    if parts.len() < 3 {
        return false;
    }
    let family = parts[0];
    let class = parts[1];
    let vcpu = parts[2];
    if family.is_empty() || class.is_empty() || vcpu.is_empty() {
        return false;
    }
    vcpu.chars().all(|c| c.is_ascii_digit())
        && family.chars().all(|c| c.is_ascii_alphanumeric())
        && class.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Validate public `CapacitySpec` admission: reject tier names, accept direct GCE types.
pub fn decide_capacity_spec(spec: &CapacitySpec) -> Result<(), Rejection> {
    if !is_valid_direct_gce_machine_type(&spec.initial_machine_type) {
        return Err(Rejection {
            reason: RejectionReason::UnsupportedMachineType,
            field_path: "initial_machine_type".to_string(),
            message: format!(
                "machine type `{}` is not an allowed direct GCE machine type; service-tier names are forbidden",
                spec.initial_machine_type
            ),
        });
    }
    Ok(())
}

/// Storage volume specification and defaults.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityStorage {
    pub size: String,
    pub storage_class: String,
    pub disk_type: String,
}

impl CapacityStorage {
    pub fn default() -> Self {
        Self {
            size: DEFAULT_DATA_STORAGE.to_string(),
            storage_class: DEFAULT_STORAGE_CLASS.to_string(),
            disk_type: DEFAULT_DISK_TYPE.to_string(),
        }
    }
}

impl Default for CapacityStorage {
    fn default() -> Self {
        Self::default()
    }
}

/// Validate storage specification for admission and online growth.
pub fn decide_storage(storage: &CapacityStorage) -> Result<(), Rejection> {
    if storage.size.is_empty() {
        return Err(Rejection {
            reason: RejectionReason::UnsupportedMachineType,
            field_path: "size".to_string(),
            message: "storage size must not be empty".to_string(),
        });
    }
    Ok(())
}

/// Resolved profile details for placement.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResolvedProfile {
    pub machine_type: String,
    pub selector: String,
    pub selector_key: String,
    pub selector_value: String,
    pub max_nodes: u32,
    pub min_nodes: u32,
    pub lifecycle_state: String,
}

/// Resolve a direct GCE machine type against a capacity catalog.
pub fn resolve_machine_type(
    machine_type: &str,
    catalog: &CapacityCatalog,
) -> Result<ResolvedProfile, Rejection> {
    let matching: Vec<&CatalogEntry> = catalog
        .entries
        .iter()
        .filter(|e| e.machine_type == machine_type)
        .collect();

    if matching.is_empty() {
        return Err(Rejection {
            reason: RejectionReason::UnsupportedMachineType,
            field_path: "machine_type".to_string(),
            message: format!("machine type `{machine_type}` is not present in capacity catalog"),
        });
    }

    if matching.len() > 1 {
        return Err(Rejection {
            reason: RejectionReason::CatalogAmbiguous,
            field_path: "catalog".to_string(),
            message: format!("multiple entries for machine type `{machine_type}` found in catalog"),
        });
    }

    let entry = matching[0];
    Ok(ResolvedProfile {
        machine_type: entry.machine_type.clone(),
        selector: entry.selector.clone(),
        selector_key: entry.stable_selector.key.clone(),
        selector_value: entry.stable_selector.value.clone(),
        max_nodes: entry.max_nodes,
        min_nodes: entry.min_nodes,
        lifecycle_state: entry.lifecycle_state.clone(),
    })
}

/// Request envelope for preflight validation.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityRequest {
    pub spec: CapacitySpec,
    pub old_member_disrupted: bool,
}

/// Preflight capacity validation: fails closed on missing, ambiguous, draining,
/// full, or incompatible catalog profiles before disrupting existing members.
pub fn preflight_capacity(
    request: &CapacityRequest,
    catalog: Option<&CapacityCatalog>,
) -> Result<ResolvedProfile, Rejection> {
    preflight_capacity_with_nodes(request, catalog, 0)
}

/// Preflight capacity validation with observable current node count for fullness checking.
pub fn preflight_capacity_with_nodes(
    request: &CapacityRequest,
    catalog: Option<&CapacityCatalog>,
    current_nodes: u32,
) -> Result<ResolvedProfile, Rejection> {
    let catalog = catalog.ok_or_else(|| Rejection {
        reason: RejectionReason::CatalogMissing,
        field_path: "catalog".to_string(),
        message: "capacity catalog is missing".to_string(),
    })?;

    let matching: Vec<&CatalogEntry> = catalog
        .entries
        .iter()
        .filter(|e| e.machine_type == request.spec.initial_machine_type)
        .collect();

    if matching.is_empty() {
        return Err(Rejection {
            reason: RejectionReason::UnsupportedMachineType,
            field_path: "machine_type".to_string(),
            message: format!(
                "machine type `{}` not found in catalog",
                request.spec.initial_machine_type
            ),
        });
    }

    if matching.len() > 1 {
        return Err(Rejection {
            reason: RejectionReason::CatalogAmbiguous,
            field_path: "catalog".to_string(),
            message: format!(
                "multiple catalog entries match machine type `{}`",
                request.spec.initial_machine_type
            ),
        });
    }

    let entry = matching[0];

    if entry.stable_selector.key.is_empty()
        || entry.stable_selector.value.is_empty()
        || (entry.lifecycle_state != "ready" && entry.lifecycle_state != "draining")
    {
        return Err(Rejection {
            reason: RejectionReason::CatalogIncompatible,
            field_path: "catalog".to_string(),
            message: format!(
                "capacity profile for `{}` is marked incompatible or invalid",
                entry.machine_type
            ),
        });
    }

    if entry.lifecycle_state == "draining" {
        return Err(Rejection {
            reason: RejectionReason::CatalogDraining,
            field_path: "catalog".to_string(),
            message: format!(
                "capacity profile for `{}` is in draining lifecycle",
                entry.machine_type
            ),
        });
    }

    if entry.max_nodes == 0 || (current_nodes > 0 && current_nodes >= entry.max_nodes) {
        return Err(Rejection {
            reason: RejectionReason::CapacityFull,
            field_path: "catalog".to_string(),
            message: format!(
                "capacity pool for `{}` is at maximum capacity (max: {}, current: {})",
                entry.machine_type, entry.max_nodes, current_nodes
            ),
        });
    }

    Ok(ResolvedProfile {
        machine_type: entry.machine_type.clone(),
        selector: entry.selector.clone(),
        selector_key: entry.stable_selector.key.clone(),
        selector_value: entry.stable_selector.value.clone(),
        max_nodes: entry.max_nodes,
        min_nodes: entry.min_nodes,
        lifecycle_state: entry.lifecycle_state.clone(),
    })
}

/// Resource dimension vector (CPU millicores, memory MiB).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityVector {
    pub cpu_millicores: u64,
    pub memory_mib: u64,
}

/// Derive pod requests by subtracting reserves and headroom from allocatable capacity.
pub fn derive_requests(
    allocatable: CapacityVector,
    reserves: CapacityVector,
    headroom: CapacityVector,
) -> Result<CapacityVector, Rejection> {
    let needed_cpu = reserves.cpu_millicores.saturating_add(headroom.cpu_millicores);
    let needed_mem = reserves.memory_mib.saturating_add(headroom.memory_mib);

    if allocatable.cpu_millicores <= needed_cpu || allocatable.memory_mib <= needed_mem {
        return Err(Rejection {
            reason: RejectionReason::InsufficientAllocatable,
            field_path: "allocatable".to_string(),
            message: format!(
                "insufficient allocatable capacity: allocatable ({}m, {}Mi) <= reserves+headroom ({}m, {}Mi)",
                allocatable.cpu_millicores, allocatable.memory_mib, needed_cpu, needed_mem
            ),
        });
    }

    Ok(CapacityVector {
        cpu_millicores: allocatable.cpu_millicores - needed_cpu,
        memory_mib: allocatable.memory_mib - needed_mem,
    })
}

/// Placement record for an instance member on a Kubernetes node.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Placement {
    pub instance: String,
    pub namespace: String,
    pub node_name: String,
}

/// Resolved shared placement for multiple instances sharing a machine type pool.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SharedPlacement {
    pub machine_type: String,
    pub selector_key: String,
    pub selector_value: String,
    pub selectors: BTreeMap<String, String>,
}

/// Resolve shared placement across instances, enforcing one data member per node.
pub fn resolve_shared_placement(
    machine_type: &str,
    catalog: &CapacityCatalog,
    placements: &[Placement],
) -> Result<SharedPlacement, Rejection> {
    let profile = resolve_machine_type(machine_type, catalog)?;

    let mut seen_nodes = BTreeSet::new();
    for p in placements {
        if !seen_nodes.insert(&p.node_name) {
            return Err(Rejection {
                reason: RejectionReason::DataMemberNodeConflict,
                field_path: "placements".to_string(),
                message: format!(
                    "duplicate placement on node `{}`; only one Lumen data member allowed per node cluster-wide",
                    p.node_name
                ),
            });
        }
    }

    let mut selectors = BTreeMap::new();
    for p in placements {
        selectors.insert(p.instance.clone(), profile.selector.clone());
    }

    Ok(SharedPlacement {
        machine_type: profile.machine_type,
        selector_key: profile.selector_key,
        selector_value: profile.selector_value,
        selectors,
    })
}

/// Transition and cluster capacity bounds policy.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityPolicy {
    #[serde(default)]
    pub allowed_transitions: Vec<String>,
    pub node_cap: u32,
    pub read_replica_cap: u32,
    pub shard_cap: u32,
    pub cooldown_seconds: u64,
}

/// Decision output for a capacity transition.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TransitionDecision {
    pub from_machine_type: String,
    pub to_machine_type: String,
    pub node_cap: u32,
    pub read_replica_cap: u32,
    pub shard_cap: u32,
    pub cooldown_seconds: u64,
}

/// Decide transition between machine types bounded by configured policy and catalog maximum.
pub fn decide_transition(
    from_machine_type: &str,
    to_machine_type: &str,
    policy: &CapacityPolicy,
    catalog_maximum: u32,
) -> Result<TransitionDecision, Rejection> {
    if from_machine_type != to_machine_type
        && !policy.allowed_transitions.iter().any(|t| t == "scale_out" || t == to_machine_type)
    {
        return Err(Rejection {
            reason: RejectionReason::TransitionNotAllowed,
            field_path: "allowed_transitions".to_string(),
            message: format!("transition `{from_machine_type}` -> `{to_machine_type}` is not permitted by policy"),
        });
    }

    let effective_node_cap = policy.node_cap.min(catalog_maximum);

    Ok(TransitionDecision {
        from_machine_type: from_machine_type.to_string(),
        to_machine_type: to_machine_type.to_string(),
        node_cap: effective_node_cap,
        read_replica_cap: policy.read_replica_cap,
        shard_cap: policy.shard_cap,
        cooldown_seconds: policy.cooldown_seconds,
    })
}

/// Operator-owned capacity lifecycle and transition state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CapacityState {
    pub current_machine_type: String,
    pub target_machine_type: String,
    pub transition_generation: u64,
    pub phase: String,
    #[serde(default)]
    pub old_member_authoritative: bool,
}

impl CapacityState {
    pub fn new(current: &str, target: &str, gen: u64, phase: &str) -> Self {
        Self {
            current_machine_type: current.to_string(),
            target_machine_type: target.to_string(),
            transition_generation: gen,
            phase: phase.to_string(),
            old_member_authoritative: true,
        }
    }
}

/// Reapplication of unchanged initial spec preserves operator-owned state without reset.
pub fn apply_capacity_reapplication(
    previous: &CapacityState,
    _spec: &CapacitySpec,
) -> CapacityState {
    previous.clone()
}

/// Project capacity status on preflight outcome or block.
pub fn project_capacity_status(
    previous: &CapacityState,
    _verdict: &Rejection,
    old_member_healthy: bool,
) -> CapacityState {
    CapacityState {
        current_machine_type: previous.current_machine_type.clone(),
        target_machine_type: previous.target_machine_type.clone(),
        transition_generation: previous.transition_generation,
        phase: "CapacityBlocked".to_string(),
        old_member_authoritative: old_member_healthy,
    }
}

/// Render cluster-wide one-member-per-node anti-affinity matching across all namespaces.
pub fn cross_namespace_dedicated_data_node_affinity() -> serde_json::Value {
    serde_json::json!({
        "podAntiAffinity": {
            "requiredDuringSchedulingIgnoredDuringExecution": [{
                "labelSelector": {
                    "matchLabels": {
                        "app.kubernetes.io/name": "lumen",
                        "app.kubernetes.io/component": "server",
                    }
                },
                "namespaceSelector": {},
                "topologyKey": "kubernetes.io/hostname",
            }]
        }
    })
}
