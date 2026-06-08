---
id: crate-unification
type: spec
title: "Crate Unification and Rename"
version: 1
spec_type: utility
spec_group: sdd
created_at: 2026-02-15T03:47:11.770024+00:00
updated_at: 2026-02-15T03:47:11.770024+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: class
      title: "Crate Unification"
changes:
  - file: crates/cclab-genesis/Cargo.toml
    action: MODIFY
  - file: crates/cclab-aurora/Cargo.toml
    action: DELETE
  - file: Cargo.toml
    action: MODIFY
history:
  - timestamp: 2026-02-15T03:47:11.770024+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Crate Unification and Rename

## Overview

This spec covers the renaming of `cclab-genesis` to `cclab-sdd` and the merging of `cclab-aurora` functionality into the new crate. This aligns the crate structure with the unified SDD vision and removes the separate Aurora generator crate. All workspace dependencies will be updated to point to `cclab-sdd`.

## Requirements

### R1 - Rename Genesis Crate

```yaml
id: R1
priority: medium
status: draft
```

Rename the `cclab-genesis` crate to `cclab-sdd` in `Cargo.toml` and directory structure.

### R2 - Merge Aurora Code

```yaml
id: R2
priority: medium
status: draft
```

Move all source code from `cclab-aurora` into `cclab-sdd/src/mcp/tools/aurora` (initially) or appropriate modules.

### R3 - Remove Aurora Crate

```yaml
id: R3
priority: medium
status: draft
```

Remove the `cclab-aurora` crate from the workspace and file system.

### R4 - Update Workspace Dependencies

```yaml
id: R4
priority: medium
status: draft
```

Update all internal crate dependencies in the workspace to replace `cclab-genesis` and `cclab-aurora` with `cclab-sdd`.

## Acceptance Criteria

### Scenario: Build Workspace

- **WHEN** running `cargo build --workspace`
- **THEN** the build should succeed without errors

### Scenario: Check Aurora Tools

- **WHEN** inspecting the `cclab-sdd` library symbols
- **THEN** the tools should be available under the `cclab-sdd` crate

## Diagrams

### Crate Unification

```mermaid
classDiagram
    class cclab-sdd {
        <<service>>
    }
    class cclab-aurora {
        <<service>>
    }
    cclab-aurora ..> cclab-sdd : merges into
```

</spec>
