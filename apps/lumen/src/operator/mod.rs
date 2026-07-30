// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! K8s Operator for lumen: a `Lumen` custom resource ([`crd`]) plus a reconcile
//! loop ([`reconcile`]) that renders ([`render`]) and applies the serving
//! data-plane. Behind the `operator` feature so the serving image never links
//! kube-rs.
//!
//! ```text
//! Lumen (lumen.dev/v1alpha1)  --reconcile-->  ServiceAccount, ConfigMap,
//!                                             Deployment/StatefulSet, Service,
//!                                             PDB,
//!                                             [ServiceMonitor, PrometheusRule]
//! ```

pub mod crd;
pub mod fleet;
pub mod lease;
pub mod reconcile;
pub mod render;
pub mod reshard_driver;
pub mod resize;

pub use crd::{Lumen, LumenSpec, LumenStatus};
pub use fleet::{LumenFleet, LumenFleetSpec, LumenFleetStatus};
pub use reconcile::run;

/// The CEL rule enforcing that a `Lumen` names at most one token source
/// (#2678, R7).
///
/// Presence tests only. Both fields render as `nullable: true`, which reads
/// like it needs an explicit `!= null` guard — it does not, and adding one
/// breaks the CRD outright. Kubernetes types a nullable string as plain
/// `string`, so `self.tokensSecret != null` fails CEL compilation at the API
/// server ("found no matching overload for '_!=_' applied to '(string,
/// null)'"), and every local test still passes because they assert on YAML
/// text, never on the compiled expression. The guard is also unnecessary:
/// Kubernetes prunes an explicitly-null field before CEL runs, so `has()`
/// already reports it absent. Verified against a live API server — with this
/// rule shape, `{tokensSecret: "x", tokensSecretProviderClass: null}` is
/// accepted and naming both is rejected.
const ONE_TOKEN_SOURCE_RULE: &str =
    "!(has(self.tokensSecret) && has(self.tokensSecretProviderClass))";

const ONE_TOKEN_SOURCE_MESSAGE: &str =
    "set at most one of spec.tokensSecret (a Kubernetes Secret) or \
     spec.tokensSecretProviderClass (a Secret Manager CSI projection); with both set there is no \
     way to tell which registry is actually being served";

/// Every CustomResourceDefinition this operator owns, as one multi-document
/// YAML: the namespaced `Lumen` data plane, then the cluster-scoped
/// [`fleet::LumenFleet`] that declares which `Lumen` objects exist.
///
/// One document rather than two files because the two are not independently
/// installable: a fleet whose `Lumen` CRD is absent applies cleanly and then
/// fails every instance, which is a worse failure than not installing.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#source
pub fn crd_yaml() -> String {
    format!("{}---\n{}", lumen_crd_yaml(), fleet::fleet_crd_yaml())
}

/// The `Lumen` CustomResourceDefinition as YAML, for `kubectl apply`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#source
pub fn lumen_crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Lumen::crd()).expect("CRD serializes to JSON");
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
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R7/AC8's schema half: the mutual exclusion is in the CRD an operator
    /// applies, so the rejection happens at `kubectl apply` rather than in a
    /// controller log nobody is reading.
    #[test]
    fn the_crd_rejects_naming_both_token_sources() {
        let yaml = crd_yaml();
        assert!(yaml.contains("x-kubernetes-validations"), "{yaml}");
        assert!(yaml.contains("tokensSecretProviderClass"), "{yaml}");
        // Presence tests only. A `!= null` guard added here would read as
        // defensive and would in fact make the CRD uninstallable — Kubernetes
        // types a nullable string as plain `string` and rejects the comparison
        // at admission, which no assertion on YAML text can catch.
        assert!(
            yaml.contains("!(has(self.tokensSecret) && has(self.tokensSecretProviderClass))"),
            "{yaml}"
        );
        assert!(!yaml.contains("!= null"), "{yaml}");
    }

    /// A fleet whose `Lumen` CRD is missing applies cleanly and then fails
    /// every instance it declares, so the two CRDs ship as one document.
    #[test]
    fn one_apply_installs_both_custom_resources() {
        let yaml = crd_yaml();
        assert!(yaml.contains("name: lumens.lumen.dev"), "{yaml}");
        assert!(yaml.contains("name: lumenfleets.lumen.dev"), "{yaml}");
        assert_eq!(
            yaml.matches("\n---\n").count(),
            1,
            "exactly one document separator, so `kubectl apply -f` sees two objects"
        );
    }

    /// R4/AC9: omitting `spec.auth` must not deploy an open API.
    #[test]
    fn auth_defaults_to_required() {
        assert_eq!(crd::AuthMode::default(), crd::AuthMode::Required);
        assert_eq!(crd::AuthMode::default().as_env(), "required");
    }
}
// CODEGEN-END
