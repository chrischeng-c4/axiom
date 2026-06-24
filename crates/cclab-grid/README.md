# Cclab Grid

## Brief

Cclab Grid is the unified Rust spreadsheet engine crate for cclab grid
workflows.

It owns the sparse sheet/workbook core, formula parser and evaluator,
undo/redo history, feature-gated database persistence, range/spatial query
helpers, and the feature-gated `cclab-grid-server` collaboration service. The
server binary compiles under the default features, but HTTP/WebSocket journey
coverage is still tracked as smoke-level evidence rather than full conformance.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Spreadsheet Core And Editing State | - | implemented | passing | conformance | not_ready | sheet/workbook data model, formatting, validation, search, selection/input/viewport state, and command history |
| Formula Evaluation Engine | - | implemented | passing | conformance | not_ready | formula parser/evaluator, functions, cross-sheet references, dependency graph, and reference shifting |
| Grid Persistence And Spatial Query | - | implemented | passing | conformance | not_ready | feature-gated Morton/WAL cell storage, query builders, and Yrs snapshot/update store |
| Collaboration Server And Workbook API | - | implemented | passing | smoke | not_ready | `cclab-grid-server` and server modules compile; HTTP/WebSocket journey coverage is still missing |

### Spreadsheet Core And Editing State

ID: spreadsheet-core-and-editing-state
Type: DeveloperTool
Surfaces:
- Rust API: `cclab_grid::core::{Sheet, Workbook, Cell, CellValue, CellRange, CellFormat, DataValidationRule, SearchEngine, SpreadsheetState}` - spreadsheet model and editing-state surface.
- Rust API: `cclab_grid::history::{HistoryManager, Command}` - undo/redo command history surface.
EC Dimensions: behavior: `cargo test -p cclab-grid` - cell values, chunked grid, sheet/workbook operations, formatting, validation, search/replace, selection, clipboard, input/edit/viewport state, and history commands
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Grid provides the Rust spreadsheet core for sparse sheets, workbooks, formatting, validation, search/replace, interactive grid state, and undo/redo command history.
Gate Inventory: `cargo test -p cclab-grid`; crates/cclab-grid/src/core; crates/cclab-grid/src/history

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Spreadsheet core and editing contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-grid`; crates/cclab-grid/src/core; crates/cclab-grid/src/history |

### Formula Evaluation Engine

ID: formula-evaluation-engine
Type: DeveloperTool
Surfaces: Rust API: `cclab_grid::formula::{NomParser, Parser, Evaluator, CrossSheetEvaluator, DependencyGraph, evaluate_formula, evaluate_formula_cross_sheet}` - formula parsing, evaluation, dependency, and reference-shift surface
EC Dimensions: behavior: `cargo test -p cclab-grid` - lexer/parser behavior, arithmetic/comparison/string/function evaluation, cross-sheet references, lookup/logical/math/date/text functions, dependency cycles, and reference shifting
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Grid parses and evaluates spreadsheet formulas, resolves cross-sheet references, tracks formula dependencies, and shifts references across row/column mutations.
Gate Inventory: `cargo test -p cclab-grid`; crates/cclab-grid/src/formula

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Formula engine behavior contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-grid`; crates/cclab-grid/src/formula |

### Grid Persistence And Spatial Query

ID: grid-persistence-and-spatial-query
Type: RuntimeTool
Surfaces:
- Cargo feature: `db` - enables the grid database module.
- Rust API: `cclab_grid::db::{CellStore, MortonKey, StoredCell, RangeQuery, SpatialQuery, YrsStore}` - storage/query/snapshot persistence surface.
EC Dimensions: behavior: `cargo test -p cclab-grid` - Morton encode/decode/order/locality, WAL write/replay, cell-store recovery/range query/delete, range/spatial query builders, and Yrs update/snapshot store operations
Root WI: -
Status: confirmed
Required Verification: conformance
Promise:
Cclab Grid persists spreadsheet cells and collaboration snapshots through a feature-gated database layer with Morton-keyed cell storage, WAL replay, range/spatial queries, and Yrs update compaction.
Gate Inventory: `cargo test -p cclab-grid`; crates/cclab-grid/src/db

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Grid database and query contract | epic | - | implemented | passing | conformance | `cargo test -p cclab-grid`; crates/cclab-grid/src/db |

### Collaboration Server And Workbook API

ID: collaboration-server-and-workbook-api
Type: Service
Surfaces:
- CLI: `cclab-grid-server` - starts the Axum workbook/collaboration server from `HOST`, `PORT`, and `DATABASE_PATH`.
- Cargo feature: `server` - enables Axum, cclab-kv storage, Yrs collaboration, REST routes, and WebSocket router.
- Rust API: `cclab_grid::server::{run_server, Config}` - server startup/config surface.
EC Dimensions: behavior: `cargo test -p cclab-grid` - server modules and binary compile under default features; smoke coverage only because no HTTP/WebSocket journey test is present
Root WI: -
Status: confirmed
Required Verification: smoke
Promise:
Cclab Grid exposes a workbook/collaboration service surface through the `cclab-grid-server` binary, environment-based server config, REST workbook routes, and a Yrs-backed collaboration WebSocket router.
Gate Inventory: `cargo test -p cclab-grid`; crates/cclab-grid/src/bin/cclab-grid-server.rs; crates/cclab-grid/src/server

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Collaboration server smoke contract | epic | - | implemented | passing | smoke | `cargo test -p cclab-grid`; crates/cclab-grid/src/bin/cclab-grid-server.rs; crates/cclab-grid/src/server |
