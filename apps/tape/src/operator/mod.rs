// HANDWRITE-BEGIN gap="missing-generator:logic:a37990fc" tracker="pending-tracker" reason="Feature-gated (operator) module root: crd/render/reconcile submodules, re-exports (Tape, TapeSpec, TapeStatus, run), and crd_yaml() uses the shared Kubernetes schema normalizer before YAML serialization."
//! K8s operator for tape: a `Tape` custom resource ([`crd`]) plus a
//! reconcile loop ([`reconcile`]) that renders ([`render`]) tape's single
//! raft-group topology — ServiceAccount, headless + client Services,
//! PodDisruptionBudget, and the downward-API StatefulSet raft-runtime consumes.
//! Behind the `operator` feature; the service image enables it because that
//! same image also runs the checked-in operator Deployment.
//!
//! ```text
//! Tape (tape.dev/v1alpha1)  --reconcile-->  ServiceAccount, StatefulSet,
//!                                           headless + client Service,
//!                                           PodDisruptionBudget
//! ```

pub mod crd;
pub mod reconcile;
pub mod render;

pub use crd::{Tape, TapeSpec, TapeStatus};
pub use reconcile::run;

/// The `Tape` CustomResourceDefinition as YAML, for `kubectl apply`.
///
/// The schema is normalized to be Kubernetes-OpenAPI compatible: schemars
/// emits `format: uint32`/`uint64` for tape's unsigned counts, which the API
/// server's structural-schema validation rejects, so those are rewritten to a
/// plain integer with a `minimum: 0` floor (relay/keep's pattern).
pub fn crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Tape::crd()).expect("CRD serializes to JSON");
    service_k8s::crd::normalize_unsigned_integer_formats(&mut crd);
    let yaml = serde_yaml::to_string(&crd).expect("CRD serializes");
    quote_kubernetes_yaml_string_defaults(&yaml)
}

/// Kubernetes still accepts YAML 1.1 input, where bare `off` is a boolean.
/// `serde_yaml` emits that Rust string as a plain scalar, so quote every
/// schema default with that spelling before the manifest reaches the API
/// server. The JSON CRD remains string-typed; this is serialization hygiene.
fn quote_kubernetes_yaml_string_defaults(yaml: &str) -> String {
    let trailing_newline = yaml.ends_with('\n');
    let mut normalized = yaml
        .lines()
        .map(|line| {
            if line.trim() == "default: off" {
                let indent = &line[..line.len() - line.trim_start().len()];
                format!(r#"{indent}default: "off""#)
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if trailing_newline {
        normalized.push('\n');
    }
    normalized
}

// HANDWRITE-END
