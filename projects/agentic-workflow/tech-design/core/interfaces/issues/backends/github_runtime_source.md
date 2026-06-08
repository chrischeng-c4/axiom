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
---

# GitHub Backend Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/issues/backends/github.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitHubBackend` | projects/agentic-workflow/src/issues/backends/github.rs | struct | pub | 27 |  |
| `new` | projects/agentic-workflow/src/issues/backends/github.rs | function | pub | 37 | new(repo: Option<String>) -> Self |
| `with_host` | projects/agentic-workflow/src/issues/backends/github.rs | function | pub | 44 | with_host(repo: Option<String>, host: Option<String>) -> Self |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap github-backend-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/issues/backends/github.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:github-backend-runtime>"
    description: "Source template owns GitHub backend runtime behavior and tests."
```
