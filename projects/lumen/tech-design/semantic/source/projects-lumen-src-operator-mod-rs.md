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

Public API manifest for `projects/lumen/src/operator/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `crd` | projects/lumen/src/operator/mod.rs | module | pub | 15 |  |
| `lease` | projects/lumen/src/operator/mod.rs | module | pub | 16 |  |
| `reconcile` | projects/lumen/src/operator/mod.rs | module | pub | 17 |  |
| `render` | projects/lumen/src/operator/mod.rs | module | pub | 18 |  |
| `reshard_driver` | projects/lumen/src/operator/mod.rs | module | pub | 19 |  |
| `resize` | projects/lumen/src/operator/mod.rs | module | pub | 20 |  |
| `crd_yaml` | projects/lumen/src/operator/mod.rs | function | pub | 26 | crd_yaml() -> String |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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
pub mod reshard_driver;
pub mod resize;

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

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #809: add `pub mod resize;` alongside the existing operator submodules,
      exposing the new raftStorage PVC resize helper.
  - path: projects/lumen/src/operator/mod.rs
    action: modify
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      #1381: add `pub mod reshard_driver;` — the autonomous reshard
      phase-driver background loop (#1319 R2 executor).
```
