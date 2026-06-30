// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! K8s Operator for lumen: a `Lumen` custom resource ([`crd`]) plus a reconcile
//! loop ([`reconcile`]) that renders ([`render`]) and applies the serving
//! data-plane. Behind the `operator` feature so the serving image never links
//! kube-rs.
//!
//! ```text
//! Lumen (lumen.dev/v1alpha1)  --reconcile-->  ServiceAccount, ConfigMap,
//!                                             Deployment/StatefulSet, Service,
//!                                             HPA when applicable, PDB,
//!                                             [ServiceMonitor, PrometheusRule]
//! ```

pub mod crd;
pub mod lease;
pub mod reconcile;
pub mod render;

pub use crd::{Lumen, LumenSpec, LumenStatus};
pub use reconcile::run;

/// The `Lumen` CustomResourceDefinition as YAML, for `kubectl apply`.
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-operator-mod-rs.md#source
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Lumen::crd()).expect("CRD serializes to JSON");
    normalize_kubernetes_schema_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

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
// CODEGEN-END
