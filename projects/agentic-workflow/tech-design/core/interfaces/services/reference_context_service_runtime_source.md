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

Public API manifest for `projects/agentic-workflow/src/services/reference_context_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateCodebaseContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 24 |  |
| `CreateContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | enum | pub | 44 |  |
| `CreateKnowledgeContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 59 |  |
| `CreateSpecContextInput` | projects/agentic-workflow/src/services/reference_context_service.rs | struct | pub | 79 |  |
| `create_context` | projects/agentic-workflow/src/services/reference_context_service.rs | function | pub | 105 | create_context(input: CreateContextInput, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:fold-shadow -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/reference_context_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:fold-shadow>"
    description: "Source template owns reference-context runtime behavior and test module link."
```
