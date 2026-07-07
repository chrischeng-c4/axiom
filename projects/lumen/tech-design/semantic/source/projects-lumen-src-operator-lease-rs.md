---
id: projects-lumen-src-operator-lease-rs
capability_refs:
  - id: "long-running-stability"
    role: primary
    claim: "kustomize-base-overlays-hpa"
    coverage: partial
    rationale: "This source unit is captured as a per-file rust-source-unit during lumen td_ast standardization."
fill_sections: [overview, source, changes]
---

# Standardized projects/lumen/src/operator/lease.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/operator/lease.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! lumen's leader-election lease — now the shared `operator::lease`.
//!
//! The implementation moved to `libs/operator` (the Lease name is parameterized
//! by the operator's field manager). lumen keeps this module as a thin re-export
//! so existing `crate::operator::lease::*` paths still resolve.

pub use operator::lease::*;
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/operator/lease.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/operator/lease.rs` captured during lumen
      standardization onto the per-file codegen ladder.
```
