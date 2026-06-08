---
id: sdd-interfaces-services-post-clarifications-service-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Post Clarifications Service Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/post_clarifications_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Contradiction` | projects/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 15 |  |
| `CreatePostClarificationsInput` | projects/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 28 |  |
| `PostClarificationsResult` | projects/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 39 |  |
| `PostQuestion` | projects/agentic-workflow/src/services/post_clarifications_service.rs | struct | pub | 50 |  |
| `create_post_clarifications` | projects/agentic-workflow/src/services/post_clarifications_service.rs | function | pub | 66 | create_post_clarifications(     input: CreatePostClarificationsInput,     project_root: &Path, ) -> Result<PostClarificationsResult> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap post-clarifications-service-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/post_clarifications_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:post-clarifications-service-runtime>"
    description: "Source template owns post-clarifications runtime behavior and tests."
```
