---
id: projects-jet-logic-legacy-pnpm-parity-md
fill_sections: [changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Jet Pnpm Parity Legacy Snapshot

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: ".aw/tech-design/projects/jet/logic/legacy/pnpm-parity.md"
    action: modify
    section: doc
    impl_mode: hand-written
    description: |
      Legacy Jet TD content retained as notes during AW standardization.
      Rewrite this file into semantic TD sections before promoting source to CODEGEN.
```

## Legacy notes
<!-- type: doc lang: markdown -->

# Jet Pnpm Parity Legacy Snapshot

### Overview

The old root `pkg-manager-pnpm-parity.md` was a broad planning snapshot for
bringing Jet closer to pnpm behavior: `.npmrc`, frozen lockfiles,
optional/platform filtering, npm aliases, overrides, workspace support, store
GC, audit, patching, and publishing.

That content is now split across active specs and source modules:

| Area | Active Contract |
|------|-----------------|
| Install flow, resolver cache, store, lockfile | `jet/logic/pkg-manager.md` |
| Workspace protocol and pnpm-workspace.yaml | `jet/logic/workspace-protocol.md` |
| Module resolver details | `jet/logic/resolver.md` |
| Package manager source | `crates/jet/src/pkg_manager/` |
| Workspace integration tests | `crates/jet/tests/workspace_protocol.rs` |

This file keeps the parity snapshot as provenance only. New changes should
update the active focused specs rather than expanding this legacy note.

### Legacy Scope

```yaml
legacy_scope:
  npmrc:
    source_modules:
      - crates/jet/src/pkg_manager/npmrc.rs
    active_spec: jet/logic/pkg-manager.md
  frozen_lockfile:
    source_modules:
      - crates/jet/src/pkg_manager/lockfile.rs
      - crates/jet/src/pkg_manager/mod.rs
    active_spec: jet/logic/pkg-manager.md
  optional_alias_overrides:
    source_modules:
      - crates/jet/src/pkg_manager/resolver.rs
      - crates/jet/src/pkg_manager/platform.rs
    active_spec: jet/logic/pkg-manager.md
  workspace_protocol:
    source_modules:
      - crates/jet/src/pkg_manager/workspace.rs
      - crates/jet/src/pkg_manager/mod.rs
    active_spec: jet/logic/workspace-protocol.md
  audit_patch_publish_gc:
    source_modules:
      - crates/jet/src/pkg_manager/audit.rs
      - crates/jet/src/pkg_manager/patch.rs
      - crates/jet/src/pkg_manager/publish.rs
      - crates/jet/src/pkg_manager/gc.rs
    active_spec: jet/logic/pkg-manager.md
```

### Changes

```yaml
changes:
  - path: .aw/tech-design/crates/jet/pkg-manager-pnpm-parity.md
    action: delete
    impl_mode: hand-written
    description: "Remove the broad root loose parity snapshot."
  - path: .aw/tech-design/crates/jet/logic/legacy/pnpm-parity.md
    action: add
    impl_mode: hand-written
    description: "Keep a short provenance pointer to active focused Jet package-manager specs."
```
