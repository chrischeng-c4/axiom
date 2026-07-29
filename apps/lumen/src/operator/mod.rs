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

/// The CEL rule enforcing that identity grants come with an audience (#2764).
///
/// Rejecting this at `kubectl apply` rather than at reconcile is the point: the
/// misconfiguration it prevents is the one that produces no error anywhere. An
/// ID-token verifier with an empty audience list accepts every token Google
/// mints for anyone, and reports each as a successful authentication.
///
/// Presence tests plus `size()`, and nothing else. A field rendered
/// `nullable: true` reads like it needs an explicit `!= null` guard — it does
/// not, and adding one breaks the CRD outright: Kubernetes types a nullable
/// string as plain `string`, so `!= null` fails CEL compilation at the API
/// server ("found no matching overload for '_!=_' applied to '(string,
/// null)'"), while every local test still passes because they assert on YAML
/// text and never on the compiled expression. The guard is also unnecessary —
/// Kubernetes prunes an explicitly-null field before CEL runs, so `has()`
/// already reports it absent. The `size()` term is not defensive padding: it
/// mirrors `LumenSpec::validate`'s `is_empty()` exactly, so an explicit
/// `identities: {}` is accepted by both lines of defence rather than one.
const IDENTITY_AUDIENCE_RULE: &str = "!(has(self.identities) && size(self.identities) > 0 && \
                                      !has(self.identityAudiences))";

const IDENTITY_AUDIENCE_MESSAGE: &str =
    "spec.identities requires at least one spec.identityAudiences entry: an ID-token verifier \
     with no audience accepts a token minted for any other Google-fronted service, and reports \
     it as a successful authentication";

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
        IDENTITY_AUDIENCE_RULE,
        IDENTITY_AUDIENCE_MESSAGE,
    );
    assert!(
        attached > 0,
        "the identity-audience rule must reach the spec schema; the generated CRD changed shape"
    );
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #2764's schema half: the audience requirement is in the CRD an operator
    /// applies, so the rejection happens at `kubectl apply` rather than in a
    /// controller log nobody is reading — and this is the one auth mistake that
    /// otherwise produces no log line at all.
    #[test]
    fn the_crd_rejects_identity_grants_with_no_audience() {
        let yaml = crd_yaml();
        assert!(yaml.contains("x-kubernetes-validations"), "{yaml}");
        assert!(yaml.contains("identityAudiences"), "{yaml}");
        assert!(yaml.contains(IDENTITY_AUDIENCE_RULE), "{yaml}");
        // Presence tests and `size()` only. A `!= null` guard added here would
        // read as defensive and would in fact make the CRD uninstallable —
        // Kubernetes types a nullable string as plain `string` and rejects the
        // comparison at admission, which no assertion on YAML text can catch.
        assert!(!yaml.contains("!= null"), "{yaml}");
    }

    /// The CSI transport is gone (#2764), and the retirement has to be visible
    /// in the artifact operators actually apply — a field left in the CRD is a
    /// field somebody sets.
    #[test]
    fn the_crd_no_longer_offers_a_csi_token_source() {
        let yaml = crd_yaml();
        assert!(!yaml.contains("tokensSecretProviderClass"), "{yaml}");
        assert!(!yaml.contains("SecretProviderClass"), "{yaml}");
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
