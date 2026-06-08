---
change_id: grid-merge-cells
type: knowledge_context
created_at: 2026-02-10T03:39:02.836017+00:00
updated_at: 2026-02-10T03:39:02.836017+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - cclab-grid
  - architecture
---

# Knowledge Context

## Relevant Documents

- **cclab/knowledge/grid-architecture.md**
  - summary: Grid architecture: Rust core -> WASM -> TS bridge -> API -> UI. Command pattern for undo/redo via History.
  - relevant sections: Command pattern, WASM bridge layer, Event emission
- **cclab/knowledge/grid-rendering.md**
  - summary: Canvas rendering pipeline, viewport management, cell rendering with format application.
  - relevant sections: Cell rendering loop, Merge-aware rendering

## Patterns

- **Command pattern for undo/redo** (source: cclab-grid-history)
  - All mutations go through Command objects (SetCellFormatCommand, SetRangeFormatCommand, etc.) stored in History for undo/redo. Merge/unmerge likely need their own commands.
- **Event emission after mutation** (source: RusheetAPI.ts)
  - After WASM mutation, RusheetAPI emits typed events. MergeCellsEvent and UnmergeCellsEvent already defined.
- **DOM overlay for UI components** (source: ContextMenu.ts, MenuBar.ts)
  - UI overlays (menus, context menus) use position:fixed elements appended to body with z-index layering.

## Pitfalls

- SetRangeFormatCommand replaces entire CellFormat instead of merging (already fixed in toolbar)
- WasmBridge.getSelection().activeCell is [number, number] tuple, NOT {row, col}
- No getCellFormat in WasmBridge - use getCellData(row, col)?.format instead
