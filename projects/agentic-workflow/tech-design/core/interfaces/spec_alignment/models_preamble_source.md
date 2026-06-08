---
id: sdd-interfaces-spec-alignment-models-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Spec Alignment Models Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/models.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CheckResult` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 15 |  |
| `CodeBlock` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 30 |  |
| `CoverageEntry` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 45 |  |
| `CoverageReport` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 59 |  |
| `FileResult` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 79 |  |
| `OrphanRequirementEntry` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 91 |  |
| `SchemaStructMismatchEntry` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 103 |  |
| `SectionAnnotation` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 117 |  |
| `SpecAnnotation` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 130 |  |
| `SpecDocument` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 146 |  |
| `SpecSection` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 158 |  |
| `UnspeccedFunction` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 175 |  |
| `Violation` | projects/agentic-workflow/src/spec_alignment/models.rs | struct | pub | 189 |  |
| `ViolationKind` | projects/agentic-workflow/src/spec_alignment/models.rs | enum | pub | 221 |  |
| `is_format_violation` | projects/agentic-workflow/src/spec_alignment/models.rs | function | pub | 251 | is_format_violation(&self) -> bool |
## Source
<!-- type: source lang: rust -->

```rust
//! Data types for spec alignment checking.
//!
//! Corresponds to the JSON Schema definitions in the check-alignment change spec:
//! SpecDocument, SpecSection, CodeBlock, Violation, ViolationKind, FileResult, CheckResult.
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/models.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns spec-alignment model module documentation."
```
