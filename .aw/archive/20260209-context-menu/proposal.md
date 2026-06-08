---
id: context-menu
type: proposal
version: 1
created_at: 2026-02-09T08:33:41.985594+00:00
updated_at: 2026-02-09T08:33:41.985594+00:00
author: mcp
status: proposed
iteration: 1
summary: "Add Google Sheets-style right-click context menu with clipboard, row/col ops, sort, and filter"
history:
  - timestamp: 2026-02-09T08:33:41.985594+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 8
  new_files: 2
affected_specs:
  - id: context-menu-ui
    path: specs/context-menu-ui.md
    depends: []
  - id: context-menu-clipboard
    path: specs/context-menu-clipboard.md
    depends: [context-menu-ui]
  - id: context-menu-operations
    path: specs/context-menu-operations.md
    depends: [context-menu-ui]
---

<proposal>

# Change: context-menu

## Summary

Add Google Sheets-style right-click context menu with clipboard, row/col ops, sort, and filter

## Why

RuSheet currently has no right-click context menu, forcing users to rely on keyboard shortcuts or toolbar buttons for common operations like insert/delete rows, copy/paste, and sorting. This is a significant UX gap compared to Google Sheets and Excel Online, where the context menu is the primary discovery mechanism for grid operations.

The backend already supports most required operations (insert/delete rows/cols, sort, filter, merge, conditional formatting, data validation) through the WASM API and RusheetAPI layer. However, these operations lack a discoverable UI entry point. Users cannot perform basic spreadsheet workflows without knowing specific keyboard shortcuts.

Adding a context menu provides an intuitive, familiar interface that surfaces existing capabilities and adds new clipboard operations. The DOM overlay approach (following FilterDropdown.ts and CellEditor.ts patterns) ensures consistent UX and maintainable code. The menu dynamically adapts based on selection context — single cell, range, row header, or column header clicks show different relevant options.

## What Changes

- Create ContextMenu.ts UI component with DOM overlay, keyboard navigation, sub-menus, and dynamic item generation based on selection context
- Add contextmenu event handling in InputController.ts to capture right-clicks on cells, row headers, and column headers
- Implement clipboard operations (Cut/Copy/Paste) using navigator.clipboard API with TSV serialization for cell ranges
- Wire existing RusheetAPI operations (insertRows, deleteRows, insertCols, deleteCols, sortRange, mergeCells, applyColumnFilter) to context menu items
- Add context-menu.css stylesheet for menu positioning, hover states, separators, disabled items, and sub-menu arrows
- Integrate ContextMenu into both main.ts (vanilla JS) and RuSheet.tsx (React) entry points

## Impact

- **Scope**: minor
- **Affected Files**: ~8
- **New Files**: ~2
- Affected specs:
  - `context-menu-ui` (no dependencies)
  - `context-menu-clipboard` → depends on: `context-menu-ui`
  - `context-menu-operations` → depends on: `context-menu-ui`
- Affected code: `src/ui/ContextMenu.ts (NEW)`, `src/styles/context-menu.css (NEW)`, `src/ui/InputController.ts`, `src/core/RusheetAPI.ts`, `src/main.ts`, `src/react/RuSheet.tsx`, `src/styles/main.css`, `src/canvas/GridRenderer.ts`

</proposal>
