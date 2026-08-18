//! K8s operator for loom: a `Loom` custom resource ([`crd`]) plus a reconcile
//! loop ([`reconcile`]) that renders ([`render`]) the raft-backed control plane,
//! and a backup runner ([`backup`]). Behind the `operator` feature so the
//! serving image never links kube-rs unless it is the operator/backup image.
//!
//! ```text
//! Loom (loom.dev/v1alpha1) --reconcile--> ServiceAccount, StatefulSet,
//!                                          Service (headless + client), PDB,
//!                                          [backup CronJob]
//! ```

pub mod backup;
pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{Loom, LoomSpec, LoomStatus};
pub use reconcile::run;

/// Server-side-apply field manager + leader-election Lease name. Per-service so
/// two operators never collide on the same Lease.
pub const MANAGER: &str = "loom-operator";

/// The `Loom` CustomResourceDefinition as YAML, for `kubectl apply`.
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Loom::crd()).expect("CRD serializes to JSON");
    normalize_kubernetes_schema_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

/// Kubernetes rejects OpenAPI `format: uint32`/`uint64` on integer schemas.
/// schemars emits them for `u32`/`u64`; strip them so the CRD is apply-clean
/// (the `#[serde]` non-negative intent is preserved by `type: integer`).
fn normalize_kubernetes_schema_formats(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if matches!(
                map.get("format").and_then(|v| v.as_str()),
                Some("uint32" | "uint64")
            ) {
                map.remove("format");
            }
            for child in map.values_mut() {
                normalize_kubernetes_schema_formats(child);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                normalize_kubernetes_schema_formats(child);
            }
        }
        _ => {}
    }
}
