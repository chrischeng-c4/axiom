---
id: sdd-interfaces-spec-alignment-format-rules-preamble-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Spec Alignment Format Rules Preamble Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/format_rules.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check` | projects/agentic-workflow/src/spec_alignment/format_rules.rs | function | pub | 63 | check(doc: &SpecDocument) -> Vec<Violation> |
## Source
<!-- type: source lang: rust -->

```rust
//! Format compliance rules for spec alignment checking.
//!
//! Three rules:
//! - `missing_section_annotation`: every `## Heading` must have an annotation
//! - `duplicate_section`: no duplicate heading text within a file
//! - `format_priority_violation`: typed sections must contain matching code blocks

use std::collections::HashMap;

#[cfg(test)]
use super::models::{CodeBlock, SectionAnnotation, SpecSection};
use super::models::{SpecDocument, Violation, ViolationKind};
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/format_rules.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<module-preamble>"
    description: "Source template owns spec-alignment format-rules documentation and imports."
```
