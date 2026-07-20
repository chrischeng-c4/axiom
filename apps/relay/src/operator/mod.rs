// HANDWRITE-BEGIN gap="missing-generator:logic:5849f96b" tracker="pending-tracker" reason="Feature-gated operator module root: crd/render/reconcile submodules, re-exports (Relay, RelaySpec, RelayStatus, run), crd_yaml() = serde_json(Relay::crd()) -> normalize_kubernetes_schema_formats (recursively rewrite format: uint32/uint64 to plain integer + minimum: 0 — Kubernetes structural-schema rules) -> serde_yaml string (keep's pattern)."
//! K8s operator for relay: a `Relay` custom resource ([`crd`]) plus a
//! reconcile loop ([`reconcile`]) that renders ([`render`]) relay's HA
//! topology — ServiceAccount, headless + client Services, PodDisruptionBudget,
//! and the downward-API StatefulSet raft-runtime consumes. Behind the `operator`
//! feature so the serving image never links kube-rs.
//!
//! ```text
//! Relay (relay.dev/v1alpha1)  --reconcile-->  ServiceAccount, StatefulSet,
//!                                             headless + client Service,
//!                                             PodDisruptionBudget
//! ```

pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{Relay, RelaySpec, RelayStatus};
pub use reconcile::run;

/// The `Relay` CustomResourceDefinition as YAML, for `kubectl apply`.
///
/// The schema is normalized to be Kubernetes-OpenAPI compatible: schemars
/// emits `format: uint32`/`uint64` for relay's unsigned counts, which the API
/// server's structural-schema validation rejects, so those are rewritten to a
/// plain integer with a `minimum: 0` floor (keep's pattern).
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Relay::crd()).expect("CRD serializes to JSON");
    service_k8s::crd::normalize_unsigned_integer_formats(&mut crd);
    let yaml = serde_yaml::to_string(&crd).expect("CRD serializes");
    service_k8s::crd::quote_yaml_1_1_boolean_like_strings(&yaml)
}
// HANDWRITE-END
