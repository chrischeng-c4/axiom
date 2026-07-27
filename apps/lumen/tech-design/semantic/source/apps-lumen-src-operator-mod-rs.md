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
| `crd` | apps/lumen/src/operator/mod.rs | module | pub | 15 |  |
| `crd_yaml` | apps/lumen/src/operator/mod.rs | function | pub | 40 | crd_yaml() -> String |
| `lease` | apps/lumen/src/operator/mod.rs | module | pub | 16 |  |
| `reconcile` | apps/lumen/src/operator/mod.rs | module | pub | 17 |  |
| `render` | apps/lumen/src/operator/mod.rs | module | pub | 18 |  |
| `reshard_driver` | apps/lumen/src/operator/mod.rs | module | pub | 19 |  |
| `resize` | apps/lumen/src/operator/mod.rs | module | pub | 20 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
pub mod lease;
pub mod reconcile;
pub mod render;
pub mod reshard_driver;
pub mod resize;

pub use crd::{Lumen, LumenSpec, LumenStatus};
pub use reconcile::run;

/// The CEL rule enforcing that a `Lumen` names at most one token source
/// (#2678, R7). Both fields render as `nullable: true`, so an explicit
/// `tokensSecret: null` is *present* to `has()` — the `!= null` guards are
/// what keep that from reading as "both are set".
const ONE_TOKEN_SOURCE_RULE: &str = "!(has(self.tokensSecret) && self.tokensSecret != null \
                                      && has(self.tokensSecretProviderClass) \
                                      && self.tokensSecretProviderClass != null)";

const ONE_TOKEN_SOURCE_MESSAGE: &str =
    "set at most one of spec.tokensSecret (a Kubernetes Secret) or \
     spec.tokensSecretProviderClass (a Secret Manager CSI projection); with both set there is no \
     way to tell which registry is actually being served";

/// The `Lumen` CustomResourceDefinition as YAML, for `kubectl apply`.
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-operator-mod-rs.md#source
pub fn crd_yaml() -> String {
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
        // Nullable fields make `has()` alone insufficient; assert the guard
        // that distinguishes "absent" from "explicitly null" survived.
        assert!(yaml.contains("self.tokensSecret != null"), "{yaml}");
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
    impl_mode: hand-written
    description: |
      #809: add `pub mod resize;` alongside the existing operator submodules,
      exposing the new raftStorage PVC resize helper.
  - path: apps/lumen/src/operator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1381: add `pub mod reshard_driver;` — the autonomous reshard
      phase-driver background loop (#1319 R2 executor).
```
