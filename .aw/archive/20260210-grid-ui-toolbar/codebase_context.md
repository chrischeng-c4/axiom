---
change_id: grid-ui-toolbar
type: codebase_context
created_at: 2026-02-10T02:45:59.584152+00:00
updated_at: 2026-02-10T02:45:59.584152+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - manual_code_review
---

# Codebase Context

## Analyzed Files

- **src/canvas/GridRenderer.ts** — Canvas renderer - needs header gridline separators added to renderHeaders()
  - symbols: `renderHeaders`, `renderGrid`, `gridToScreen`, `screenToGrid`, `setScrollOffset`
- **src/canvas/theme.ts** — Theme constants for rendering - headerHeight, headerWidth, gridLineColor
  - symbols: `theme`
- **index.html** — Main HTML structure - toolbar and formula bar need complete redesign
  - symbols: `toolbar`, `formula-bar`, `cell-address`, `formula-input`
- **src/styles/main.css** — Global CSS - toolbar/formula-bar styles need Google Sheets-like redesign
  - symbols: `.toolbar`, `.formula-bar`, `.toolbar-btn`, `.toolbar-separator`
- **src/core/RusheetAPI.ts** — Core API singleton - provides setRangeFormat, undo, redo, mergeCells, event emitter
  - symbols: `rusheet`, `setRangeFormat`, `setCellValue`, `onSelectionChange`, `emitSelectionChanged`, `undo`, `redo`
- **src/core/WasmBridge.ts** — WASM bridge - low-level format getters/setters, selection, history
  - symbols: `getSelection`, `setRangeFormat`, `getCellFormat`, `canUndo`, `canRedo`, `undo`, `redo`
- **src/ui/InputController.ts** — Event handling - keyboard shortcuts for undo/redo/copy/paste already defined
  - symbols: `handleKeyDown`, `syncSelectionToRenderer`
- **src/ui/ContextMenu.ts** — Reference pattern for DOM overlay menus with keyboard nav
  - symbols: `ContextMenu`, `MenuItem`, `show`, `hide`, `buildItems`
- **src/ui/FilterDropdown.ts** — Reference pattern for dropdown positioned relative to grid
  - symbols: `FilterDropdown`, `show`, `hide`
- **src/react/RuSheet.tsx** — React wrapper component - layout container for toolbar/formula-bar/canvas/statusbar
  - symbols: `RuSheet`, `useEffect`, `forwardRef`, `useImperativeHandle`
- **src/main.ts** — Vanilla JS entry point - wires up toolbar buttons, formula bar, keyboard events
  - symbols: `initGrid`, `setupToolbar`, `setupFormulaBar`
- **src/types/index.ts** — TypeScript types including CellFormat for styling
  - symbols: `CellFormat`, `CellData`, `SelectionState`
- **crates/cclab-grid-wasm/src/api.rs** — Rust WASM API - set_range_format, get_cell_format exposed to JS
  - symbols: `set_range_format`, `get_cell_format`, `get_selection`
- **crates/cclab-grid-core/src/lib.rs** — Rust grid core - CellFormat struct definition
  - symbols: `CellFormat`, `Grid`, `set_range_format`

## Prism Results

- **manual_code_review** (query: `GridRenderer.renderHeaders header gridlines`)
  - renderHeaders() at lines 481-560 draws header backgrounds and text but only 2 border lines (vertical after row headers, horizontal after column headers). Missing: vertical separators between each column header and horizontal separators between each row header.
- **manual_code_review** (query: `RusheetAPI formatting API surface`)
  - RusheetAPI exposes setRangeFormat(startRow, startCol, endRow, endCol, format, source), getCellFormat via WasmBridge.getCellFormat(row,col), undo/redo, mergeCells/unmergeCells. CellFormat includes bold, italic, underline, fontSize, textColor, backgroundColor, horizontalAlign, verticalAlign.
- **manual_code_review** (query: `index.html current toolbar structure`)
  - Current toolbar contains only Save/Load/Export buttons and Autocomplete checkbox. Formula bar has cell-address span and formula-input text field. Both need complete redesign to match Google Sheets.

## Dependency Graph

- index.html -> src/main.ts -> src/core/WasmBridge.ts -> crates/cclab-grid-wasm
- src/main.ts -> src/canvas/GridRenderer.ts -> src/canvas/theme.ts
- src/main.ts -> src/ui/InputController.ts -> src/core/RusheetAPI.ts
- src/main.ts -> src/ui/ContextMenu.ts -> src/core/RusheetAPI.ts
- src/react/RuSheet.tsx -> src/core/RusheetAPI.ts -> src/core/WasmBridge.ts
- NEW: src/ui/Toolbar.ts -> src/core/RusheetAPI.ts (formatting commands)
- NEW: src/ui/MenuBar.ts -> src/ui/ContextMenu.ts pattern (dropdown menus)
