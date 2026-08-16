---
id: projects-lumen-src-operator-mod-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/operator/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/operator/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `certificate` | apps/lumen/src/operator/mod.rs | module | pub | 15 |  |
| `crd` | apps/lumen/src/operator/mod.rs | module | pub | 16 |  |
| `crd_yaml` | apps/lumen/src/operator/mod.rs | function | pub | 55 | crd_yaml() -> String |
| `fleet` | apps/lumen/src/operator/mod.rs | module | pub | 17 |  |
| `lease` | apps/lumen/src/operator/mod.rs | module | pub | 18 |  |
| `reconcile` | apps/lumen/src/operator/mod.rs | module | pub | 19 |  |
| `render` | apps/lumen/src/operator/mod.rs | module | pub | 20 |  |
| `reshard_driver` | apps/lumen/src/operator/mod.rs | module | pub | 21 |  |
| `resize` | apps/lumen/src/operator/mod.rs | module | pub | 22 |  |
## Source
<!-- type: rust-source-unit lang: rust -->



````rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! K8s Operator for lumen: a `Lumen` custom resource ([`crd`]), plus a reconcile loop ([`reconcile`])
//! that renders ([`render`]) and applies the serving data-plane.
//!
//! The operator consumes externally provisioned TLS Secrets; it does not own
//! certificate issuer configuration or lifecycle.
//!
//! ```text
//! Lumen (lumen.dev/v1alpha1)  --reconcile-->  ServiceAccount, ConfigMap,
//!                                             Deployment/StatefulSet, Service,
//!                                             PDB,
//!                                             [ServiceMonitor, PrometheusRule]
//! ```

#[cfg(feature = "operator")]
pub mod capacity;
#[cfg(feature = "operator")]
pub mod certificate;
#[cfg(feature = "operator")]
pub mod crd;
#[cfg(feature = "operator")]
pub mod fleet;
#[cfg(feature = "operator")]
pub mod lease;
#[cfg(feature = "operator")]
pub mod reconcile;
#[cfg(feature = "operator")]
pub mod render;
#[cfg(feature = "operator")]
pub mod reshard_driver;
#[cfg(feature = "operator")]
pub mod resize;

#[cfg(feature = "operator")]
pub use crd::{Lumen, LumenSpec, LumenStatus};
#[cfg(feature = "operator")]
pub use fleet::{LumenFleet, LumenFleetSpec, LumenFleetStatus};
#[cfg(feature = "operator")]
pub use reconcile::run;

/// The CEL operator no CRD rule here may use, kept as a written rule because
/// its absence is not self-evident from the schema (#2764, #2872).
///
/// A field rendered `nullable: true` reads like it needs an explicit
/// `!= null` guard. It does not, and adding one breaks the CRD outright:
/// Kubernetes types a nullable string as plain `string`, so `!= null` fails CEL
/// compilation at the API server ("found no matching overload for '_!=_'
/// applied to '(string, null)'") — while every local test still passes, because
/// they assert on YAML text and never on the compiled expression. The guard is
/// also unnecessary: Kubernetes prunes an explicitly-null field before CEL runs,
/// so `has()` already reports it absent. Presence tests plus `size()`, nothing
/// else.
///
/// Test-only because there is no rule left to apply it to. It stays at module
/// scope, next to [`lumen_crd_yaml`], so the next author to add one reads the
/// rule before writing the expression rather than after the cluster rejects it.
#[cfg(test)]
#[cfg(feature = "operator")]
const FORBIDDEN_CEL_OPERATOR: &str = "!= null";

/// Every CustomResourceDefinition this operator owns, as one multi-document
/// YAML: the namespaced `Lumen` data plane, then the cluster-scoped
/// [`fleet::LumenFleet`] that declares which `Lumen` objects exist.
///
/// One document rather than two files because the two are not independently
/// installable: a fleet whose `Lumen` CRD is absent applies cleanly and then
/// fails every instance, which is a worse failure than not installing.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#source
#[cfg(feature = "operator")]
pub fn crd_yaml() -> String {
    format!("{}---\n{}", lumen_crd_yaml(), fleet::fleet_crd_yaml())
}

/// The `Lumen` CustomResourceDefinition as YAML, for `kubectl apply`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#source
#[cfg(feature = "operator")]
pub fn lumen_crd_yaml() -> String {
    use kube::CustomResourceExt;
    let mut crd = serde_json::to_value(crd::Lumen::crd()).expect("CRD serializes to JSON");
    service_k8s::crd::normalize_unsigned_integer_formats(&mut crd);
    serde_yaml::to_string(&crd).expect("CRD serializes")
}

#[cfg(all(test, feature = "operator"))]
mod tests {
    use super::*;

    /// #2872 AC3. The identity-audience rule was the only CEL rule this CRD
    /// carried, and it guarded a field that no longer exists — so the check
    /// that survives is the one on the *shape* of any rule added later.
    ///
    /// `!= null` cannot be caught by asserting on rendered YAML text (see
    /// [`FORBIDDEN_CEL_OPERATOR`]); it only fails at the API server. This test
    /// is the local half. `kubectl apply --dry-run=server` is the other.
    #[test]
    fn no_surviving_cel_rule_names_a_retired_field_or_uses_a_forbidden_operator() {
        let yaml = crd_yaml();
        for retired in ["identityAudiences", "identities", "tokensSecret"] {
            assert!(
                !yaml.contains(retired),
                "retired field `{retired}` must not survive in the CRD: {yaml}"
            );
        }
        assert!(!yaml.contains(FORBIDDEN_CEL_OPERATOR), "{yaml}");
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/operator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Canonical lossless source unit for the Lumen operator module registry.
      Historical #809 and #1381 additions, plus later operator modules, are
      regenerated together from the unique rust-source-unit above.
```
