---
id: grid-ui-toolbar
type: proposal
version: 1
created_at: 2026-02-10T02:51:02.248600+00:00
updated_at: 2026-02-10T02:51:02.248600+00:00
author: mcp
status: proposed
iteration: 1
summary: "Implement Google Sheets-style toolbar, menu bar, formula bar, and header gridline separators"
history:
  - timestamp: 2026-02-10T02:51:02.248600+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 12
  new_files: 4
affected_specs:
  - id: header-gridlines
    path: specs/header-gridlines.md
    depends: []
  - id: toolbar-formatting
    path: specs/toolbar-formatting.md
    depends: [header-gridlines]
  - id: menu-bar-dropdowns
    path: specs/menu-bar-dropdowns.md
    depends: []
  - id: formula-bar-redesign
    path: specs/formula-bar-redesign.md
    depends: []
---

<proposal>

# Change: grid-ui-toolbar

## Summary

Implement Google Sheets-style toolbar, menu bar, formula bar, and header gridline separators

## Why

The current Grid UI lacks essential spreadsheet interaction points. Users have no way to format cells (bold, italic, colors, alignment), access application menus, or easily distinguish headers from data due to missing gridline separators between column headers and row headers. The existing toolbar contains only Save/Load/Export buttons, and the formula bar is a basic text input without proper styling.

This limitation severely restricts usability compared to standard tools like Google Sheets. Users expect a formatting toolbar with undo/redo, font controls, text styling, colors, borders, merge, and alignment buttons - all wired to the existing WASM formatting API (setRangeFormat, getCellFormat). They also expect a menu bar with File/Edit/View/Insert/Format/Data dropdowns containing actionable items.

Addressing these gaps is critical for user adoption and establishes the UI foundation for advanced features. The WASM backend already supports the needed formatting operations; this change bridges the gap between backend capability and user-facing controls.

## What Changes

- Add header gridline separators between each column header and each row header in GridRenderer.ts renderHeaders()
- Create Toolbar component with formatting buttons (undo/redo, font, size, bold/italic/underline/strikethrough, text color, fill color, borders, merge, alignment) wired to WASM API
- Create MenuBar component with File/Edit/View/Insert/Format/Data dropdown menus containing actionable items
- Redesign formula bar with cell address dropdown, fx icon, and styled input matching Google Sheets
- Integrate toolbar state sync with onSelectionChange events to reflect active cell formatting
- Update index.html structure and main.css styles for the new layout

## Impact

- **Scope**: minor
- **Affected Files**: ~12
- **New Files**: ~4
- Affected specs:
  - `header-gridlines` (no dependencies)
  - `toolbar-formatting` → depends on: `header-gridlines`
  - `menu-bar-dropdowns` (no dependencies)
  - `formula-bar-redesign` (no dependencies)
- Affected code: `src/canvas/GridRenderer.ts`, `src/canvas/theme.ts`, `src/ui/Toolbar.ts (new)`, `src/ui/MenuBar.ts (new)`, `src/styles/toolbar.css (new)`, `src/styles/menu-bar.css (new)`, `src/styles/main.css`, `index.html`, `src/main.ts`, `src/core/RusheetAPI.ts`, `src/core/WasmBridge.ts`, `src/types/index.ts`

</proposal>
