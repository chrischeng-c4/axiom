---
id: consolidate-codegen-spec
main_spec_ref: "crates/sdd/logic/codegen-consolidation.md"
merge_strategy: new
filled_sections: [overview, requirements, changes]
fill_sections: [overview, requirements, changes]
create_complete: true
---

# Consolidate Codegen Spec

## Overview

Move cclab-compass `generate/` module (spec_ir, generators, engine) into sdd crate. Merge duplicate SpecIR type systems. Rename `@score/*` packages to `@sdd/*`. After this change, sdd owns all codegen infrastructure and cclab-compass only keeps code intelligence (syntax, semantic, LSP, lint).
## Requirements

| ID | Title | Description | Priority |
|----|-------|-------------|----------|
| R1 | Move generate/ module | Copy cclab-compass/src/generate/{spec_ir,generators,engine,lib.rs} → sdd/src/generate/. Update mod.rs imports. | P0 |
| R2 | Merge SpecIR types | Unify compass SpecIR (10 variants) + sdd SpecManifest (6 kinds) into single type in sdd. Remove sdd spec_ir/types.rs SpecKind duplication. | P0 |
| R3 | Merge Generator traits | Replace sdd ManifestGenerator with compass SpecIRGenerator+Generator. Remove sdd spec_ir/codegen.rs duplication. | P0 |
| R4 | Move Tera engine | sdd owns TemplateEngine directly (from compass generate/engine/). | P0 |
| R5 | Rename @score → @sdd | git mv crates/sdd/packages/@score/* → crates/sdd/packages/@sdd/*. Update package.json names, pnpm-workspace, Conductor FE imports. | P1 |
| R6 | Remove compass generate/ | Delete cclab-compass/src/generate/ after move. Update compass lib.rs and Cargo.toml. | P0 |
| R7 | Wire score gen code | Update score CLI codegen.rs to use sdd's generators directly (no bridge needed). | P1 |
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

```yaml
changes:
  # R1: Move generate/ module
  - file: crates/sdd/src/generate/
    action: create
    description: Copy from cclab-compass/src/generate/ (spec_ir, generators, engine, lib.rs)
  - file: crates/sdd/src/lib.rs
    action: modify
    description: Add pub mod generate

  # R2+R3: Merge types
  - file: crates/sdd/src/spec_ir/types.rs
    action: modify
    description: Remove SpecKind/SpecManifest, re-export from generate/spec_ir
  - file: crates/sdd/src/spec_ir/codegen.rs
    action: modify
    description: Remove ManifestGenerator/ManifestDispatcher, use generate/generators/common

  # R4: Tera engine now in sdd
  - file: crates/sdd/Cargo.toml
    action: modify
    description: Add tera, heck deps if not present

  # R5: Rename @score → @sdd
  - file: crates/sdd/packages/@sdd/
    action: create
    description: git mv from @score/
  - file: pnpm-workspace.yaml
    action: modify
    description: Update package paths
  - file: projects/conductor/fe/package.json
    action: modify
    description: Update @score → @sdd imports

  # R6: Remove from compass
  - file: crates/cclab-compass/src/generate/
    action: delete
    description: Entire generate/ module removed
  - file: crates/cclab-compass/src/lib.rs
    action: modify
    description: Remove pub mod generate
  - file: crates/cclab-compass/Cargo.toml
    action: modify
    description: Remove tera dep if only used by generate/

  # R7: Wire CLI
  - file: projects/score/cli/src/codegen.rs
    action: modify
    description: Use sdd::generate directly
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
