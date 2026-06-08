---
change_id: grid-select-range
type: codebase_context
created_at: 2026-02-09T06:24:29.555345+00:00
updated_at: 2026-02-09T06:24:29.555345+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - "prism_symbols (via gemini:flash agent)"
  - file reads for all critical files
---

# Codebase Context

## Analyzed Files

- **crates/cclab-grid-core/src/state/selection.rs** — Core selection model - has Selection struct with primary/additional ranges, active_cell, and methods for extend_to, add_range, select_row/column/all, is_selected, normalize
  - symbols: `CellPosition`, `SelectionRange`, `SelectionMode`, `Selection`, `select_cell`, `extend_to`, `add_range`, `select_row`, `select_column`, `select_all`, `is_selected`, `normalize`
- **crates/cclab-grid-core/src/range.rs** — Range types (CellCoord, CellRange) with A1 notation parsing, contains, intersects, iter - used for formatting/validation/merging but NOT for selection
  - symbols: `CellCoord`, `CellRange`, `from_a1`, `to_a1`, `contains`, `intersects`, `iter`, `row_count`, `col_count`, `cell_count`
- **crates/cclab-grid-core/src/spatial.rs** — Spatial index with Fenwick trees for efficient row/col offset lookups - used for rendering coordinates
  - symbols: `SpatialIndex`, `find_row_at_offset`, `find_col_at_offset`, `get_row_offset`, `get_col_offset`
- **crates/cclab-grid-core/src/state/mod.rs** — SpreadsheetState - top-level state management including selection field
  - symbols: `SpreadsheetState`, `Selection`
- **crates/cclab-grid-wasm/src/api.rs** — SpreadsheetEngine WASM API - exposes cell/format/row/col operations but NO selection methods currently
  - symbols: `SpreadsheetEngine`, `setCellValue`, `getCellData`, `clearRange`, `setRangeFormat`, `mergeCells`
- **crates/cclab-grid-wasm/src/viewport.rs** — Viewport management for visible area rendering
  - symbols: `Viewport`
- **src/ui/InputController.ts** — User input handling - mousedown selects single cell, arrow keys navigate, no drag or shift support
  - symbols: `InputController`, `handleMouseDown`, `handleKeyDown`, `gridRenderer.setActiveCell`, `gridRenderer.screenToGrid`
- **src/canvas/GridRenderer.ts** — Canvas rendering - tracks single activeCell, renderSelection draws border for one cell only
  - symbols: `GridRenderer`, `activeCell`, `setActiveCell`, `getActiveCell`, `renderSelection`, `screenToGrid`, `CellPosition`
- **src/core/RusheetAPI.ts** — Public API - tracks single cell selection {row, col}, emits SelectionChangeEvent
  - symbols: `RusheetAPI`, `currentSelection`, `setSelection`, `getSelection`, `onSelectionChange`, `SelectionChangeEvent`
- **src/core/WasmBridge.ts** — WASM bridge - no selection methods exposed, handles cell ops, format, undo/redo
  - symbols: `WasmBridge`
- **src/react/RuSheet.tsx** — React component - formula bar + canvas + sheet tabs, shows single cell address (A1), no status bar
  - symbols: `RuSheet`, `cellAddress`, `formulaInputRef`

## Prism Results

- **gemini:flash agent exploration** (query: `selection|range in crates/cclab-grid-core`)
  - Found 596 matches. Selection model is mature in Rust core (Selection, SelectionRange, CellPosition, extend_to, add_range, etc.) but NOT exposed via WASM API.
- **gemini:flash agent exploration** (query: `mouse|click|drag in src/`)
  - Found 71 matches. InputController handles mousedown for single cell selection and arrow keys for navigation. No mousemove/mouseup drag handling.
- **gemini:flash agent exploration** (query: `StatusBar|footer in src/`)
  - Found 6 matches. RuSheet has sheet tabs at bottom but no status bar for aggregation display.

## Dependency Graph

- selection.rs -> state/mod.rs (Selection is a field of SpreadsheetState)
- state/mod.rs -> api.rs (SpreadsheetEngine wraps SpreadsheetState for WASM)
- api.rs -> WasmBridge.ts (WASM bridge calls SpreadsheetEngine methods)
- WasmBridge.ts -> RusheetAPI.ts (API layer wraps bridge)
- RusheetAPI.ts -> InputController.ts (input events call API selection methods)
- RusheetAPI.ts -> GridRenderer.ts (renderer reads selection state to draw)
- GridRenderer.ts -> RuSheet.tsx (React component hosts canvas)
- range.rs is independent - used for formatting/merging but could unify with selection types
