---
id: sdd-interfaces-services-init-change-service-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Init Change Service Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/init_change_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateChangeInput` | projects/agentic-workflow/src/services/init_change_service.rs | struct | pub | 15 |  |
| `CreateChangeResult` | projects/agentic-workflow/src/services/init_change_service.rs | struct | pub | 28 |  |
| `create_change` | projects/agentic-workflow/src/services/init_change_service.rs | function | pub | 48 | create_change(input: CreateChangeInput, project_root: &Path) -> Result<CreateChangeResult> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap init-change-service-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/init_change_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:init-change-service-runtime>"
    description: "Source template owns init-change runtime behavior and tests."
```
