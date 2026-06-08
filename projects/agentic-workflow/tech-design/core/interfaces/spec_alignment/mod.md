---
id: projects-sdd-src-spec-alignment-mod-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/mod.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `annotations` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 12 |  |
| `check` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 13 |  |
| `coverage` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 14 |  |
| `format_rules` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 15 |  |
| `logical_rules` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 16 |  |
| `models` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 17 |  |
| `parser` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 18 |  |
| `requirement_coverage` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 19 |  |
| `schema_struct` | projects/agentic-workflow/src/spec_alignment/mod.rs | module | pub | 20 |  |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/mod.rs -->
```rust
//! Spec alignment checking.
//!
//! Validates spec files for format compliance and logical consistency.
//! Two-layer validation:
//! - Format compliance: section annotations, duplicates, code block requirements
//! - Logical consistency: duplicate definitions, schema conflicts, field near-matches
//!
//! Entry point: `spec_alignment::check(path)`.

pub mod annotations;
pub mod check;
pub mod coverage;
pub mod format_rules;
pub mod logical_rules;
pub mod models;
pub mod parser;
pub mod requirement_coverage;
pub mod schema_struct;

pub use check::{check, check_with_coverage};
pub use models::{
    CheckResult, CodeBlock, CoverageEntry, CoverageReport, FileResult, OrphanRequirementEntry,
    SchemaStructMismatchEntry, SpecAnnotation, SpecDocument, SpecSection, UnspeccedFunction,
    Violation, ViolationKind,
};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete spec-alignment module facade.
```
