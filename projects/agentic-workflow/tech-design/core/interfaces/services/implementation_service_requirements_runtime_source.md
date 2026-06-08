---
id: sdd-interfaces-services-implementation-service-requirements-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Workflow service interfaces support TD/CB artifact lifecycle authoring, review, and implementation steps."
---

# Implementation Service Requirements Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/implementation_service.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CreateMergeReviewInput` | projects/agentic-workflow/src/services/implementation_service.rs | struct | pub | 261 |  |
| `CreateReviewInput` | projects/agentic-workflow/src/services/implementation_service.rs | struct | pub | 289 |  |
| `MergeQuality` | projects/agentic-workflow/src/services/implementation_service.rs | enum | pub | 309 |  |
| `MergeReviewIssue` | projects/agentic-workflow/src/services/implementation_service.rs | struct | pub | 318 |  |
| `MergeReviewVerdict` | projects/agentic-workflow/src/services/implementation_service.rs | enum | pub | 328 |  |
| `ReviewIssue` | projects/agentic-workflow/src/services/implementation_service.rs | struct | pub | 337 |  |
| `ReviewVerdict` | projects/agentic-workflow/src/services/implementation_service.rs | enum | pub | 355 |  |
| `Severity` | projects/agentic-workflow/src/services/implementation_service.rs | enum | pub | 364 |  |
| `TestResults` | projects/agentic-workflow/src/services/implementation_service.rs | struct | pub | 373 |  |
| `create_merge_review` | projects/agentic-workflow/src/services/implementation_service.rs | function | pub | 601 | create_merge_review(input: CreateMergeReviewInput, project_root: &Path) -> Result<String> |
| `create_review` | projects/agentic-workflow/src/services/implementation_service.rs | function | pub | 435 | create_review(input: CreateReviewInput, project_root: &Path) -> Result<String> |
| `list_changed_files` | projects/agentic-workflow/src/services/implementation_service.rs | function | pub | 137 | list_changed_files(     change_id: &str,     base_branch: Option<&str>,     filter: Option<&str>,     _project_root: &Path, ) -> Result<String> |
| `read_all_requirements` | projects/agentic-workflow/src/services/implementation_service.rs | function | pub | 57 | read_all_requirements(change_id: &str, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap implementation-service-requirements-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/implementation_service.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:implementation-service-requirements-runtime>"
    description: "Source template owns implementation requirements and changed-files runtime."
```
