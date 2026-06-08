---
change_id: context-menu
type: codebase_context
created_at: 2026-02-09T08:26:52.981949+00:00
updated_at: 2026-02-09T08:26:52.981949+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - manual_exploration
---

# Codebase Context

## Analyzed Files

- **src/ui/InputController.ts** — Primary: add contextmenu event listener, detect right-click context (cell/row-header/col-header)
  - symbols: `InputController`, `handleMouseDown`, `handleMouseMove`, `handleMouseUp`, `handleKeyDown`, `isOnFilterButton`
- **src/ui/FilterDropdown.ts** — Reference pattern: DOM overlay positioning, outside-click handling, boundary checking
  - symbols: `FilterDropdown`, `show`, `hide`, `handleOutsideClick`, `createContainer`
- **src/ui/CellEditor.ts** — Reference pattern: DOM overlay lifecycle, absolute positioning over canvas, z-index layering
  - symbols: `CellEditor`, `activate`, `deactivate`, `positionTextarea`, `updatePosition`
- **src/core/RusheetAPI.ts** — API layer: all context menu operations route through here (insertRows, deleteRows, insertCols, deleteCols, sortRange, clearRange, mergeCells, unmergeCells, applyColumnFilter)
  - symbols: `rusheet`, `insertRows`, `deleteRows`, `insertCols`, `deleteCols`, `sortRange`, `clearRange`, `setCellFormat`, `setRangeFormat`, `mergeCells`, `unmergeCells`, `applyColumnFilter`, `clearColumnFilter`
- **src/core/WasmBridge.ts** — WASM bridge: direct bindings to Rust operations, used by RusheetAPI
  - symbols: `WasmBridge`, `insertRows`, `deleteRows`, `insertCols`, `deleteCols`, `sortRange`, `clearRange`, `mergeCells`, `unmergeCells`
- **src/canvas/GridRenderer.ts** — Coordinate conversion: gridToScreen/screenToGrid for positioning context menu
  - symbols: `GridRenderer`, `gridToScreen`, `screenToGrid`, `isOnFilterButton`, `getActiveCell`
- **crates/cclab-grid-wasm/src/api.rs** — WASM API: all grid operations exposed to JS (insert/delete/sort/filter/merge/validation/conditional-formatting)
  - symbols: `SpreadsheetEngine`, `insertRows`, `deleteRows`, `insertCols`, `deleteCols`, `sortRange`, `mergeCells`, `unmergeCells`, `addConditionalFormatting`, `addDataValidation`
- **src/main.ts** — Entry point: wires ContextMenu into the application, similar to how CellEditor and InputController are initialized
  - symbols: `main`
- **src/react/RuSheet.tsx** — React entry: needs parallel context menu integration for React usage
  - symbols: `RuSheet`

## Prism Results

- **manual_exploration** (query: `context menu related symbols and patterns`)
  - No existing contextmenu event handling found. FilterDropdown.ts provides excellent DOM overlay pattern. All WASM operations for insert/delete/sort/filter/merge already exist in api.rs. Clipboard operations are the only missing backend feature.

## Dependency Graph

- InputController → ContextMenu (new): dispatches contextmenu events
- ContextMenu (new) → RusheetAPI: calls operations (insertRows, deleteRows, etc.)
- ContextMenu (new) → GridRenderer: uses gridToScreen for positioning
- ContextMenu (new) → WasmBridge: reads selection state for menu item enablement
- main.ts → ContextMenu (new): instantiation and wiring
- RuSheet.tsx → ContextMenu (new): React integration
