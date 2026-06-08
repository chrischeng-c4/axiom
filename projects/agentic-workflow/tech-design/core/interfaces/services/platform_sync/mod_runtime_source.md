---
id: sdd-interfaces-services-platform-sync-mod-runtime-source
fill_sections: [overview, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Platform Sync Module Runtime Source

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/services/platform_sync/mod.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PlatformSyncService` | projects/agentic-workflow/src/services/platform_sync/mod.rs | struct | pub | 38 |  |
| `load_config` | projects/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 52 | load_config(&self) -> Result<PlatformConfig> |
| `new` | projects/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 48 | new(project_root: PathBuf) -> Self |
| `payload` | projects/agentic-workflow/src/services/platform_sync/mod.rs | module | pub | 21 |  |
| `sync` | projects/agentic-workflow/src/services/platform_sync/mod.rs | function | pub | 58 | sync(&self, change_id: &str) -> Result<SyncResult> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap platform-sync-service-runtime -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/mod.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:platform-sync-service-runtime>"
    description: "Source template owns platform sync service runtime behavior."
```
