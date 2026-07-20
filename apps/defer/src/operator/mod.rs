// HANDWRITE-BEGIN gap="missing-generator:logic:defer-operator" tracker="#766" reason="Feature-gated Defer operator adapter over service-k8s."
pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{Defer, DeferBackupSpec, DeferSpec, DeferStatus};
pub use reconcile::run;

pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(Defer::crd()).expect("CRD JSON");
    service_k8s::crd::normalize_unsigned_integer_formats(&mut crd);
    let yaml = serde_yaml::to_string(&crd).expect("CRD YAML");
    service_k8s::crd::quote_yaml_1_1_boolean_like_strings(&yaml)
}
// HANDWRITE-END
