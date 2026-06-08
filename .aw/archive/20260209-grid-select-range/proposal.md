---
id: grid-select-range
type: proposal
version: 1
created_at: 2026-02-09T06:25:51.041473+00:00
updated_at: 2026-02-09T06:25:51.041473+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add drag-to-select, keyboard range, multi-selection, and status bar aggregation to grid"
history:
  - timestamp: 2026-02-09T06:25:51.041473+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 11
  new_files: 0
affected_specs:
  - id: selection-wasm-api
    path: specs/selection-wasm-api.md
    depends: []
  - id: selection-ui-interaction
    path: specs/selection-ui-interaction.md
    depends: [selection-wasm-api]
  - id: selection-rendering
    path: specs/selection-rendering.md
    depends: [selection-wasm-api]
  - id: selection-status-bar
    path: specs/selection-status-bar.md
    depends: [selection-wasm-api, selection-rendering]
---

<proposal>

# Change: grid-select-range

## Summary

Add drag-to-select, keyboard range, multi-selection, and status bar aggregation to grid

## Why

The grid currently only supports single-cell selection via click or arrow key navigation. This severely limits the user experience for common spreadsheet operations like formatting multiple cells, copying ranges, and quick data analysis.

The Rust core already has a mature Selection model (Selection, SelectionRange, CellPosition) with support for range extension, multi-selection, row/column selection, and normalization. However, none of this is exposed through the WASM API or wired into the TypeScript frontend.

Standard spreadsheet applications (Excel, Google Sheets) offer drag-to-select, Shift+Click range extension, Shift+Arrow keyboard extension, Ctrl+Click multi-selection, and a status bar showing aggregation results (SUM, AVG, COUNT) for the selected range. Adding these features will bring the grid UX to industry standard while leveraging the existing backend infrastructure.

Performance considerations are critical: large selections (100k+ cells) must only render visible cells, mouse drag listeners must be properly cleaned up to avoid memory leaks, and selection highlight rendering must coexist with existing cell border styling.

## What Changes

- Expose Rust Selection model through SpreadsheetEngine WASM API (setSelection, getSelection, extendSelection, addSelection)
- Add WasmBridge.ts selection bridge methods for TS-WASM communication
- Implement drag-to-select in InputController.ts with mousedown/mousemove/mouseup state machine
- Add Shift+Arrow keyboard range extension and Ctrl+Click multi-selection to InputController.ts
- Update GridRenderer.ts to render selection range highlights (semi-transparent background) and multi-selection borders
- Extend RusheetAPI.ts SelectionChangeEvent to include full range info (startRow, startCol, endRow, endCol, additional ranges)
- Add status bar to RuSheet.tsx displaying SUM, AVG, COUNT, MIN, MAX for selected numeric cells

## Impact

- **Scope**: minor
- **Affected Files**: ~11
- **New Files**: ~0
- Affected specs:
  - `selection-wasm-api` (no dependencies)
  - `selection-ui-interaction` → depends on: `selection-wasm-api`
  - `selection-rendering` → depends on: `selection-wasm-api`
  - `selection-status-bar` → depends on: `selection-wasm-api`, `selection-rendering`
- Affected code: `crates/cclab-grid-core/src/state/selection.rs`, `crates/cclab-grid-wasm/src/api.rs`, `crates/cclab-grid-wasm/src/viewport.rs`, `src/core/WasmBridge.ts`, `src/core/RusheetAPI.ts`, `src/ui/InputController.ts`, `src/canvas/GridRenderer.ts`, `src/react/RuSheet.tsx`

</proposal>
