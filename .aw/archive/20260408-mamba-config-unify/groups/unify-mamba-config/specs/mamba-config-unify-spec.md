---
id: mamba-config-unify-spec
main_spec_ref: "crates/mamba/config/config-schema.md"
merge_strategy: new
filled_sections: [overview, changes]
fill_sections: [overview, changes]
create_complete: true
---

# Mamba Config Unify Spec

## Overview

Unify two duplicate `MambaConfig` structs into one canonical definition.

### Problem

`driver/config.rs` and `config/schema.rs` both define a `MambaConfig` struct.
The driver variant is simpler (entry_point, flat crates/expose maps) and is
actively used by the compiler session, CLI, and all tests.  The schema variant
is richer (ProjectConfig, CrateConfig, BuildConfig, PathsConfig) but is only
used internally within `config/schema.rs` tests.

### Solution

1. Promote `config/schema.rs::MambaConfig` as the single canonical struct.
2. Add an `entry_point` field to `ProjectConfig` (used by driver and CLI).
3. Migrate `discover()` and `is_symbol_exposed()` methods from the driver
   variant into the canonical struct.
4. Remove the `MambaConfig` struct from `driver/config.rs`.
5. Update `driver/mod.rs` to re-export from `crate::config` instead of
   `driver::config`.
6. Update `main.rs` imports and all test helpers.
## Requirements
<!-- type: requirements lang: markdown -->

<!-- TODO -->

## Scenarios
<!-- type: scenarios lang: markdown -->

<!-- TODO -->

## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/mamba/src/config/schema.rs
    action: MODIFY
    targets:
      - type: struct
        name: MambaConfig
        change: add discover(), entry_point(), is_symbol_exposed(), has_crate_expose() methods migrated from driver/config.rs
      - type: struct
        name: ProjectConfig
        change: add entry_point field, derive Default, add serde(default) on all fields
    do_not_touch: [from_file, validate]
  - path: crates/mamba/src/driver/config.rs
    action: MODIFY
    targets:
      - type: struct
        name: MambaConfig
        change: remove duplicate struct and all its impl methods; replace with re-export from crate::config::schema
  - path: crates/cclab-cli/src/mamba.rs
    action: MODIFY
    targets:
      - type: impl
        name: CliModule for MambaCli
        change: update entry_point field access to use entry_point() method returning Option
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
