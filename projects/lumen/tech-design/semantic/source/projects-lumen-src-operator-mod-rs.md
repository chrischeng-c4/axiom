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

# Standardized projects/lumen/src/operator/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/mod.rs` captured as a per-file rust-source-unit (td_ast) during lumen standardization onto the per-file codegen ladder.

### Symbols

| Name | Target | Kind | Visibility |
|------|--------|------|------------|
| `crd` | projects/lumen/src/operator/mod.rs | module | pub |
| `lease` | projects/lumen/src/operator/mod.rs | module | pub |
| `reconcile` | projects/lumen/src/operator/mod.rs | module | pub |
| `render` | projects/lumen/src/operator/mod.rs | module | pub |
| `crd_yaml` | projects/lumen/src/operator/mod.rs | function | pub |

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/operator/mod.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
