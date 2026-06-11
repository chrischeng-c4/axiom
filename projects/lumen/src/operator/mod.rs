// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-src-operator-mod-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! K8s Operator for lumen: a `Lumen` custom resource ([`crd`]) plus a reconcile
//! loop ([`reconcile`]) that renders ([`render`]) and applies the serving fleet
//! and NATS broker. Behind the `operator` feature so the serving image never
//! links kube-rs.
//!
//! ```text
//! Lumen (lumen.dev/v1alpha1)  --reconcile-->  ServiceAccount, ConfigMap,
//!                                             Deployment, Service, HPA, PDB,
//!                                             [NATS StatefulSet/Services/CM],
//!                                             [ServiceMonitor, PrometheusRule]
//! ```

pub mod crd;
pub mod lease;
pub mod reconcile;
pub mod render;

pub use crd::{Lumen, LumenSpec, LumenStatus};
pub use reconcile::run;

/// The `Lumen` CustomResourceDefinition as YAML, for `kubectl apply`.
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    serde_yaml::to_string(&crd::Lumen::crd()).expect("CRD serializes")
}

// </HANDWRITE>
