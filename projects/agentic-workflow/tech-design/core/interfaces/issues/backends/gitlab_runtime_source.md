---
id: sdd-interfaces-issues-backends-gitlab-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
---

# GitLab Backend Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/gitlab.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitLabBackend` | projects/agentic-workflow/src/issues/backends/gitlab.rs | struct | pub | 28 |  |
| `new` | projects/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 38 | new(repo: Option<String>) -> Self |
| `with_host` | projects/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 45 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap gitlab-backend-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/gitlab.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:gitlab-backend-runtime>"
    description: "Source template owns GitLab backend runtime behavior and tests."
```
