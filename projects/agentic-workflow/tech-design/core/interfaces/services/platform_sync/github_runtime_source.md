---
id: sdd-interfaces-services-platform-sync-github-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Platform Sync GitHub Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/platform_sync/github.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `GitHubProvider` | projects/agentic-workflow/src/services/platform_sync/github.rs | struct | pub | 17 |  |
| `can_sync` | projects/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 43 | can_sync(&self) -> bool |
| `new` | projects/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 29 | new(config: PlatformConfig) -> Self |
| `sync` | projects/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 63 | sync(&self, payload: &SyncPayload) -> Result<SyncResult> |
| `with_token` | projects/agentic-workflow/src/services/platform_sync/github.rs | function | pub | 37 | with_token(mut self, project_root: &std::path::Path) -> Result<Self> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap platform-sync-github-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/github.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:platform-sync-github-runtime>"
    description: "Source template owns platform sync GitHub provider runtime and tests."
```
