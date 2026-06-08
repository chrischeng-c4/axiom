---
id: projects-sdd-src-generate-spec-ir-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/spec_ir/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/spec_ir/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/spec_ir/mod.rs -->
```rust
//! SpecIR — Specification Intermediate Representation
//!
//! The universal contract between SDD generate (spec format) and Lens (code generation).
//! SpecIR wraps diagram and schema types into a unified enum that
//! generators can consume via `can_generate()` / `generate_from_ir()`.
//!
//! ## Variants
//!
//! | Variant | Section type | Generator |
//! |---------|-------------|-----------|
//! | `Api` | `rest-api` / `schema` | `FastAPIGenerator`, `ExpressGenerator`, `AxumGenerator` |
//! | `FlowchartPlus` | `logic` (flowchart) | — |
//! | `ClassPlus` | `logic` (class) | — |
//! | `ErdPlus` | `db-model` | — |
//! | `SequencePlus` | `interaction` | — |
//! | `RequirementPlus` | `test-plan` | `TestGenerator` |
//! | `Deploy` | `deploy` | `DeployGenerator` |
//! | `Wireframe` | `wireframe` | `ReactGenerator` |
//! | `Component` | `component` | — (future) |
//! | `DesignToken` | `design-token` | — (future) |

mod types;

pub use types::*;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/spec_ir/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete SpecIR module facade.
```
