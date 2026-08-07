---
id: sdd-interfaces-services-implementation-service-review-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Implementation Service Review Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/services/implementation_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateMergeReviewInput` | apps/agentic-workflow/src/services/implementation_service.rs | struct | pub | 262 |  |
| `CreateReviewInput` | apps/agentic-workflow/src/services/implementation_service.rs | struct | pub | 290 |  |
| `MergeQuality` | apps/agentic-workflow/src/services/implementation_service.rs | enum | pub | 310 |  |
| `MergeReviewIssue` | apps/agentic-workflow/src/services/implementation_service.rs | struct | pub | 319 |  |
| `MergeReviewVerdict` | apps/agentic-workflow/src/services/implementation_service.rs | enum | pub | 329 |  |
| `ReviewIssue` | apps/agentic-workflow/src/services/implementation_service.rs | struct | pub | 338 |  |
| `ReviewVerdict` | apps/agentic-workflow/src/services/implementation_service.rs | enum | pub | 356 |  |
| `Severity` | apps/agentic-workflow/src/services/implementation_service.rs | enum | pub | 365 |  |
| `TestResults` | apps/agentic-workflow/src/services/implementation_service.rs | struct | pub | 374 |  |
| `create_merge_review` | apps/agentic-workflow/src/services/implementation_service.rs | function | pub | 602 | create_merge_review(input: CreateMergeReviewInput, project_root: &Path) -> Result<String> |
| `create_review` | apps/agentic-workflow/src/services/implementation_service.rs | function | pub | 436 | create_review(input: CreateReviewInput, project_root: &Path) -> Result<String> |
| `list_changed_files` | apps/agentic-workflow/src/services/implementation_service.rs | function | pub | 138 | list_changed_files(     change_id: &str,     base_branch: Option<&str>,     filter: Option<&str>,     project_root: &Path, ) -> Result<String> |
| `read_all_requirements` | apps/agentic-workflow/src/services/implementation_service.rs | function | pub | 58 | read_all_requirements(change_id: &str, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap implementation-service-review-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/services/implementation_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:implementation-service-review-runtime>"
    description: "Source template owns implementation review runtime behavior and tests."
```
