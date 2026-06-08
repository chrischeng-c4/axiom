---
change_id: context-menu
type: knowledge_context
created_at: 2026-02-09T08:24:07.527679+00:00
updated_at: 2026-02-09T08:24:07.527679+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - grid
  - 05-titan
  - 30-claude
  - 40-mcp
  - changelogs
  - orbit
  - root
---

# Knowledge Context

## Relevant Documents

- **grid/formula-syntax.md**
  - summary: Formula syntax guide covering cell references, operators, functions (SUM, VLOOKUP, etc.), wildcards, and array formulas. Relevant for sort operations that need to understand formula references and paste operations that may involve formula adjustment.
  - relevant sections: Functions, Wildcards, Array Formulas

## Patterns

- **Event emitter pattern** (source: RusheetAPI.ts (codebase convention))
  - All user-facing operations route through RusheetAPI singleton which wraps WasmBridge calls with event emission. Context menu operations must follow this pattern - call rusheet.xxx() not WasmBridge.xxx() directly.
- **DOM overlay for UI components** (source: CellEditor.ts (codebase convention))
  - CellEditor uses absolutely-positioned DOM elements over the canvas for text editing. Context menu should follow the same pattern - HTML/CSS overlay positioned relative to the canvas container.
- **InputController interaction routing** (source: InputController.ts (codebase convention))
  - InputController handles all mouse/keyboard events on the canvas and delegates to RusheetAPI. Right-click contextmenu event should be captured here and dispatched to the context menu component.

## Pitfalls

- Clipboard API (navigator.clipboard) requires secure context (HTTPS or localhost) and user gesture
- Context menu must call preventDefault() on contextmenu event to suppress browser default menu
- Insert/delete row/col operations must update formula references (e.g., =A2 becomes =A3 after inserting a row above)
- Sort operations on filtered data should only sort visible rows
- Paste special with formula adjustment needs to handle relative vs absolute references
