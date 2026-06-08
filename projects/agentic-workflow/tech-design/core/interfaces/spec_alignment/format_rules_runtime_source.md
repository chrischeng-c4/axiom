---
id: sdd-interfaces-spec-alignment-format-rules-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Spec Alignment Format Rules Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/format_rules.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check` | projects/agentic-workflow/src/spec_alignment/format_rules.rs | function | pub | 63 | check(doc: &SpecDocument) -> Vec<Violation> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap spec-alignment-format-rules-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/format_rules.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:spec-alignment-format-rules-runtime>"
    description: "Source template owns spec-alignment format-rule runtime behavior and tests."
```
