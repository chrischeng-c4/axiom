// HANDWRITE-BEGIN gap="missing-generator:logic:a37990fc" tracker="pending-tracker" reason="Feature-gated (operator) module root: crd/render/reconcile submodules, re-exports (AuthMode, Tape, TapeSpec, TapeStatus, run), and crd_yaml() uses the shared Kubernetes schema normalizer plus the one-token-source CEL rule before YAML serialization."
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

pub use crd::{AuthMode, Tape, TapeBackupSpec, TapeSpec, TapeStatus};
pub use reconcile::run;

/// The CEL rule enforcing that a `Tape` names at most one token source
/// (#2765).
///
/// Presence tests only. Both fields render as `nullable: true`, which reads
/// like it needs an explicit `!= null` guard — it does not, and adding one
/// breaks the CRD outright. Kubernetes types a nullable string as plain
/// `string`, so `self.tokensSecret != null` fails CEL compilation at the API
/// server ("found no matching overload for '_!=_' applied to '(string,
/// null)'"), and every local test still passes because they assert on YAML
/// text, never on the compiled expression. The guard is also unnecessary:
/// Kubernetes prunes an explicitly-null field before CEL runs, so `has()`
/// already reports it absent. Lumen shipped the `!= null` shape once and it
/// installed on no cluster; it was caught only by
/// `kubectl apply --dry-run=server`. The contract is documented at
/// `libs/service-k8s/src/crd.rs`.
const ONE_TOKEN_SOURCE_RULE: &str =
    "!(has(self.tokensSecret) && has(self.tokensSecretProviderClass))";

const ONE_TOKEN_SOURCE_MESSAGE: &str =
    "set at most one of spec.tokensSecret (a Kubernetes Secret) or \
     spec.tokensSecretProviderClass (a Secret Manager CSI projection); with both set there is no \
     way to tell which registry is actually being served";

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
    let attached = service_k8s::crd::add_spec_validation_rule(
        &mut crd,
        ONE_TOKEN_SOURCE_RULE,
        ONE_TOKEN_SOURCE_MESSAGE,
    );
    assert!(
        attached > 0,
        "the one-token-source rule must reach the spec schema; the generated CRD changed shape"
    );
    let yaml = serde_yaml::to_string(&crd).expect("CRD serializes");
    service_k8s::crd::quote_yaml_1_1_boolean_like_strings(&yaml)
}

// HANDWRITE-END
