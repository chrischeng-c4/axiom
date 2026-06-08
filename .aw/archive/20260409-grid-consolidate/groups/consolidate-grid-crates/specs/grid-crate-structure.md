---
id: grid-crate-structure
main_spec_ref: crates/cclab-grid/logic/crate-structure.md
merge_strategy: new
fill_sections: [overview, changes]
filled_sections: [overview, changes]
create_complete: true
---

# Grid Crate Structure

## Overview

<!-- type: overview lang: markdown -->

Consolidate 5 separate cclab-grid-* crates (core, formula, history, db, server) into a single `cclab-grid` crate with sub-modules. The `cclab-grid-wasm` crate remains separate as it requires a `cdylib` build target for WASM compilation.

**Current structure** (6 crates, ~23k LOC):
- `cclab-grid-core` (11k LOC) — sparse matrix storage, cell types, coordinate system, range operations, sheet management
- `cclab-grid-formula` (5.8k LOC) — formula parser, evaluator, function library, dependency graph
- `cclab-grid-history` (1.9k LOC) — command-based undo/redo history
- `cclab-grid-db` (1.9k LOC) — Morton encoding persistence, WAL-backed storage, range queries
- `cclab-grid-server` (0.9k LOC) — Axum web server, CRDT collaboration, WebSocket handlers
- `cclab-grid-wasm` (2.2k LOC) — WASM bindings (stays separate)

**Target structure** (single crate + wasm):
- `cclab-grid` — unified crate with `pub mod core`, `pub mod formula`, `pub mod history`, `pub mod db`, `pub mod server`
- `cclab-grid-wasm` — unchanged, depends on `cclab-grid` instead of 3 separate crates

**Key decisions**:
- Server binary (`cclab-grid-server`) preserved as `[[bin]]` target inside `cclab-grid`
- Heavy dependencies feature-gated: `server` feature (axum, tower, yrs, etc.), `db` feature (cclab-wal, bincode, parking_lot)
- Cross-crate `use cclab_grid_core::*` imports become intra-crate `use crate::core::*`
- All public re-exports from each sub-module's `mod.rs` preserved for API compatibility
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

changes:
  # === New consolidated crate ===
  - path: crates/cclab-grid/Cargo.toml
    action: CREATE
    description: |
      Merged Cargo.toml combining deps from all 5 crates.
      Features: 'server' (axum, tower, yrs, etc.), 'db' (cclab-wal, bincode, parking_lot).
      [[bin]] target for cclab-grid-server.

  - path: crates/cclab-grid/src/lib.rs
    action: CREATE
    description: |
      Root lib.rs declaring pub mod core, pub mod formula, pub mod history.
      Conditionally: pub mod db (cfg feature db), pub mod server (cfg feature server).

  # === Core module (from cclab-grid-core) ===
  - path: crates/cclab-grid/src/core/mod.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/lib.rs. All pub mod and pub use statements preserved.

  - path: crates/cclab-grid/src/core/cell.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/cell.rs, no import changes needed.

  - path: crates/cclab-grid/src/core/chunk.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/chunk.rs.

  - path: crates/cclab-grid/src/core/conditional_format.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/conditional_format.rs.

  - path: crates/cclab-grid/src/core/error.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/error.rs.

  - path: crates/cclab-grid/src/core/format.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/format.rs.

  - path: crates/cclab-grid/src/core/gap_buffer.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/gap_buffer.rs.

  - path: crates/cclab-grid/src/core/range.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/range.rs.

  - path: crates/cclab-grid/src/core/search.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/search.rs.

  - path: crates/cclab-grid/src/core/sheet.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/sheet.rs.

  - path: crates/cclab-grid/src/core/spatial.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/spatial.rs.

  - path: crates/cclab-grid/src/core/state/mod.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/mod.rs.

  - path: crates/cclab-grid/src/core/state/clipboard.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/clipboard.rs.

  - path: crates/cclab-grid/src/core/state/edit.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/edit.rs.

  - path: crates/cclab-grid/src/core/state/input.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/input.rs.

  - path: crates/cclab-grid/src/core/state/selection.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/selection.rs.

  - path: crates/cclab-grid/src/core/state/viewport.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/state/viewport.rs.

  - path: crates/cclab-grid/src/core/validation.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/validation.rs.

  - path: crates/cclab-grid/src/core/workbook.rs
    action: CREATE
    description: Moved from cclab-grid-core/src/workbook.rs.

  # === Formula module (from cclab-grid-formula) ===
  - path: crates/cclab-grid/src/formula/mod.rs
    action: CREATE
    description: |
      Moved from cclab-grid-formula/src/lib.rs.
      Replace 'use cclab_grid_core::' with 'use crate::core::'.

  - path: crates/cclab-grid/src/formula/ast.rs
    action: CREATE
    description: Moved from cclab-grid-formula/src/ast.rs. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/dependency.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/evaluator.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/lexer.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/parser.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/parser_nom.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/reference_shifter.rs
    action: CREATE
    description: Moved from cclab-grid-formula/src/reference_shifter.rs.

  - path: crates/cclab-grid/src/formula/functions/mod.rs
    action: CREATE
    description: Moved from cclab-grid-formula/src/functions/mod.rs.

  - path: crates/cclab-grid/src/formula/functions/datetime.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/functions/logical.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/functions/lookup.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/functions/math.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/formula/functions/text.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  # === History module (from cclab-grid-history) ===
  - path: crates/cclab-grid/src/history/mod.rs
    action: CREATE
    description: |
      Moved from cclab-grid-history/src/lib.rs.
      No import changes (only re-exports).

  - path: crates/cclab-grid/src/history/command.rs
    action: CREATE
    description: |
      Moved from cclab-grid-history/src/command.rs.
      Replace cclab_grid_core:: with crate::core::.
      Replace cclab_grid_formula:: with crate::formula::.

  - path: crates/cclab-grid/src/history/stack.rs
    action: CREATE
    description: |
      Moved from cclab-grid-history/src/stack.rs.
      Replace cclab_grid_core:: with crate::core::.

  # === DB module (from cclab-grid-db) ===
  - path: crates/cclab-grid/src/db/mod.rs
    action: CREATE
    description: |
      Moved from cclab-grid-db/src/lib.rs.
      Replace cclab_grid_core:: with crate::core::.
      Gated behind cfg(feature = "db").

  - path: crates/cclab-grid/src/db/query/mod.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/query/mod.rs.

  - path: crates/cclab-grid/src/db/query/range.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/query/range.rs.

  - path: crates/cclab-grid/src/db/query/spatial.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/query/spatial.rs.

  - path: crates/cclab-grid/src/db/snapshot/mod.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/snapshot/mod.rs.

  - path: crates/cclab-grid/src/db/snapshot/store.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/snapshot/store.rs.

  - path: crates/cclab-grid/src/db/storage/mod.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/storage/mod.rs.

  - path: crates/cclab-grid/src/db/storage/cell_store.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  - path: crates/cclab-grid/src/db/storage/morton.rs
    action: CREATE
    description: Moved from cclab-grid-db/src/storage/morton.rs.

  - path: crates/cclab-grid/src/db/storage/wal.rs
    action: CREATE
    description: Moved. Replace cclab_grid_core:: with crate::core::.

  # === Server module (from cclab-grid-server) ===
  - path: crates/cclab-grid/src/server/mod.rs
    action: CREATE
    description: |
      Moved from cclab-grid-server/src/lib.rs.
      Replace cclab_grid_db:: with crate::db::.
      Gated behind cfg(feature = "server").

  - path: crates/cclab-grid/src/server/api/mod.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/api/mod.rs.

  - path: crates/cclab-grid/src/server/api/health.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/api/health.rs.

  - path: crates/cclab-grid/src/server/api/workbooks.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/api/workbooks.rs.

  - path: crates/cclab-grid/src/server/collab/mod.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/collab/mod.rs.

  - path: crates/cclab-grid/src/server/collab/document.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/collab/document.rs.

  - path: crates/cclab-grid/src/server/collab/websocket.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/collab/websocket.rs.

  - path: crates/cclab-grid/src/server/config.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/config.rs.

  - path: crates/cclab-grid/src/server/db/mod.rs
    action: CREATE
    description: Moved. Replace cclab_grid_db:: with crate::db::.

  - path: crates/cclab-grid/src/server/db/models.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/db/models.rs.

  - path: crates/cclab-grid/src/server/error.rs
    action: CREATE
    description: Moved from cclab-grid-server/src/error.rs.

  - path: crates/cclab-grid/src/bin/cclab-grid-server.rs
    action: CREATE
    description: |
      Moved from cclab-grid-server/src/main.rs.
      Replace 'use cclab_grid_server::' with 'use cclab_grid::server::'.

  # === Workspace and external crate updates ===
  - path: Cargo.toml
    action: MODIFY
    targets:
      - type: function
        name: workspace.members
        change: |
          Remove: crates/cclab-grid-core, crates/cclab-grid-formula, crates/cclab-grid-history,
          crates/cclab-grid-db, crates/cclab-grid-server.
          Add: crates/cclab-grid.
    do_not_touch: []

  - path: crates/cclab-grid-wasm/Cargo.toml
    action: MODIFY
    targets:
      - type: struct
        name: dependencies
        change: |
          Remove: cclab-grid-core, cclab-grid-formula, cclab-grid-history.
          Add: cclab-grid = { path = "../cclab-grid" }.

  - path: crates/cclab-grid-wasm/src/api.rs
    action: MODIFY
    targets:
      - type: function
        name: all_imports
        change: |
          Replace 'use cclab_grid_core::' with 'use cclab_grid::core::'.
          Replace 'use cclab_grid_formula::' with 'use cclab_grid::formula::'.
          Replace 'use cclab_grid_history::' with 'use cclab_grid::history::'.

  # === Deleted old crates ===
  - path: crates/cclab-grid-core/
    action: DELETE
    description: Entire crate directory removed after migration.

  - path: crates/cclab-grid-formula/
    action: DELETE
    description: Entire crate directory removed after migration.

  - path: crates/cclab-grid-history/
    action: DELETE
    description: Entire crate directory removed after migration.

  - path: crates/cclab-grid-db/
    action: DELETE
    description: Entire crate directory removed after migration.

  - path: crates/cclab-grid-server/
    action: DELETE
    description: Entire crate directory removed after migration.
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
