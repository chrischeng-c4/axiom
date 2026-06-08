---
id: sdd-services-platform-sync-mod
fill_sections: [overview, schema, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: client-boundary-model
    claim: client-boundary-model
    coverage: full
    rationale: "Service interfaces expose AW Core project, issue, and platform boundary behavior to clients."
---

# Platform Sync Service Type

## Overview
<!-- type: overview lang: markdown -->

Service struct for the platform-sync orchestrator in
`projects/agentic-workflow/src/services/platform_sync/mod.rs`. One shape:

- `PlatformSyncService` — single private `project_root: PathBuf`
  field with no derives. The struct is a thin handle holding the
  workspace root; all behaviour lives on the hand-written
  `impl PlatformSyncService` block (`new`, `load_config`, `sync`).

Codegen replaces the struct declaration. Companion source templates own the
module documentation, submodule declarations, re-exports, imports, constructor,
config loading, provider dispatch, and frontmatter writeback.

This spec exercises:

1. **No-derive struct emission** — `x-rust-struct.derive: []` emits
   `pub struct PlatformSyncService { ... }` with no `#[derive(...)]`
   attribute.
2. **`x-rust-visibility: private`** on the only field — keeps
   `project_root: PathBuf` private (no `pub`) on a public struct,
   matching the source.
3. **`x-rust-type: "PathBuf"`** in `required:` — uses the bare type
   without Option auto-wrapping.

## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  PlatformSyncService:
    type: object
    required: [project_root]
    description: |
      Platform sync service handle. Holds the project root path used
      to resolve `.aw/config.toml` and change directories. All
      behaviour is on the hand-written impl block.
    properties:
      project_root:
        type: string
        x-rust-type: "PathBuf"
        x-rust-visibility: private
        description: "Absolute path to the project root."
    x-rust-struct:
      derive: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/services/platform_sync/mod.rs
    action: modify
    section: schema
    impl_mode: codegen
    replaces:
      - PlatformSyncService
    description: |
      Codegen replaces the struct declaration only.
```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Correctly identifies the single struct, its private field, and lack of derives. Hand-written boundary (impl block + module-level items) is clearly stated.
- [schema] Definition is well-formed: `x-rust-struct.derive: []` for no-derive emission, `project_root` listed in `required:` with `x-rust-type: "PathBuf"` to skip Option auto-wrap, and `x-rust-visibility: private` to keep the field non-`pub`. Matches source semantics exactly.
- [changes] Two entries split codegen vs hand-written cleanly. `replaces` lists the single struct name; hand-written entry covers all module-level items and the impl block.
