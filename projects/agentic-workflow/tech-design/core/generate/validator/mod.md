---
id: projects-sdd-src-generate-validator-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Standardized projects/agentic-workflow/src/generate/validator/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/validator/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/validator/mod.rs -->
```rust
//! Spec Completeness Validator
//!
//! Validates JSON Schemas and SpecIR payloads for completeness before code generation.
//!
//! ## Modules
//!
//! - [`completeness`] — JSON Schema type/ref/description validation (R1–R3)
//! - [`spec_ir_validator`] — SpecIR section-type validators (deploy, wireframe,
//!   component, design-token) with shared registration mechanism

mod completeness;
mod spec_ir_validator;

pub use completeness::{validate_schema, Severity, ValidationIssue, ValidationResult};
pub use spec_ir_validator::{
    validate_spec_ir, ComponentValidator, DeployValidator, DesignTokenValidator, SpecIRValidator,
    WireframeValidator,
};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/validator/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete validator module facade.
```
