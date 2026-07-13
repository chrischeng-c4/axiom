---
id: sdd-interfaces-issues-backends-github-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Issue backend interfaces implement the AW Core client boundary for projecting workflow state to configured issue platforms."
  - id: project-local-td-and-ec-gates
    role: primary
    gap: project-label-producer-td-routing
    claim: project-label-producer-td-routing
    coverage: partial
    rationale: "GitHub WI updates must honor explicit stale project-label removals while preserving unrelated remote labels."
---

# GitHub Backend Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/src/issues/backends/github.rs` generated from AST during Score force-regeneration standardization.

`IssueBackend::write` remains conservative for unmanaged remote labels;
`IssueBackend::update` selects the explicit-removal write variant so requested
unmanaged removals are included in the same REST label-set patch. A resulting
empty label set is encoded with `gh api -F labels[]`; omitting the labels field
continues to mean that labels are unchanged.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitHubBackend` | apps/agentic-workflow/src/issues/backends/github.rs | struct | pub | 27 |  |
| `new` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 37 | new(repo: Option<String>) -> Self |
| `with_host` | apps/agentic-workflow/src/issues/backends/github.rs | function | pub | 44 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap github-backend-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/issues/backends/github.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:github-backend-runtime>"
    description: "Source template owns GitHub backend runtime behavior and tests, including authoritative IssuePatch label removals and explicit empty-array encoding."
```
