---
id: sdd-interfaces-issues-backends-gitlab-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: agent-first-cli-product-model
    claim: agent-first-cli-product-model
    coverage: full
    rationale: "Issue backend interfaces project the single AW CLI workflow state to configured issue platforms."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: project-label-producer-td-routing
    claim: project-label-producer-td-routing
    coverage: partial
    rationale: "GitLab WI updates must honor explicit stale project-label removals while preserving unrelated remote labels."
---

# GitLab Backend Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/issues/backends/gitlab.rs` generated from AST during Score force-regeneration standardization.

`IssueBackend::write` remains conservative for unmanaged remote labels;
`IssueBackend::update` selects the explicit-removal write variant so requested
unmanaged removals become `glab issue update --unlabel` arguments. Read
fixtures normalize every legacy non-epic type label to canonical `change`
output without mutating the original remote label set.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitLabBackend` | apps/agentic-workflow/src/issues/backends/gitlab.rs | struct | pub | 28 |  |
| `new` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 38 | new(repo: Option<String>) -> Self |
| `with_host` | apps/agentic-workflow/src/issues/backends/gitlab.rs | function | pub | 45 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap gitlab-backend-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/backends/gitlab.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:gitlab-backend-runtime>"
    description: "Source template owns GitLab backend runtime behavior and tests, including legacy type compatibility reads and authoritative IssuePatch label removals."
```
