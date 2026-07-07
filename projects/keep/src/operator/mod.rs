//! K8s operator for keep: a `Keep` custom resource ([`crd`]) plus a reconcile
//! loop ([`reconcile`]) that renders ([`render`]) keep's sharded HA topology —
//! ServiceAccount, ConfigMap, headless + client Services, PodDisruptionBudget,
//! and the downward-API StatefulSet raft-host consumes. Behind the `operator`
//! feature so the serving image never links kube-rs.
//!
//! ```text
//! Keep (keep.dev/v1alpha1)  --reconcile-->  ServiceAccount, ConfigMap,
//!                                           StatefulSet, headless + client
//!                                           Service, PodDisruptionBudget
//! ```
//!
//! @spec projects/keep/tech-design/interfaces/cli/adopt-libs-operator-keep-k8s-crd-operator-instance-cli.md

pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{Keep, KeepBackupSpec, KeepSpec, KeepStatus};
pub use reconcile::run;

/// The `Keep` CustomResourceDefinition as YAML, for `kubectl apply`.
///
/// The schema is normalized to be Kubernetes-OpenAPI compatible: schemars emits
/// `format: uint32`/`uint64` for keep's unsigned counts, which the API server's
/// structural-schema validation rejects, so those are rewritten to a plain
/// integer with a `minimum: 0` floor.
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Keep::crd()).expect("CRD serializes to JSON");
    normalize_kubernetes_schema_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

/// Recursively rewrite unsigned-int formats (`uint32`/`uint64`) — which are not
/// in the Kubernetes structural-schema format vocabulary — to a plain integer
/// with a `minimum: 0` floor, so the generated CRD applies cleanly.
fn normalize_kubernetes_schema_formats(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if matches!(
                map.get("format").and_then(|v| v.as_str()),
                Some("uint32" | "uint64")
            ) {
                map.remove("format");
                map.entry("minimum").or_insert_with(|| serde_json::json!(0));
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
