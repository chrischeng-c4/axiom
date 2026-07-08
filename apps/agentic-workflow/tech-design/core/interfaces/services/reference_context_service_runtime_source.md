---
id: sdd-interfaces-services-reference-context-service-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Reference Context Service Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/services/reference_context_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateCodebaseContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 24 |  |
| `CreateContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | enum | pub | 44 |  |
| `CreateKnowledgeContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 59 |  |
| `CreateSpecContextInput` | apps/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 79 |  |
| `create_context` | apps/agentic-workflow/src/services/reference_context_service.rs | function | pub | 105 | create_context(input: CreateContextInput, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/reference_context_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns reference-context runtime behavior and test module link."
```
