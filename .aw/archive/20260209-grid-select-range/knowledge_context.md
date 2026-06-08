---
change_id: grid-select-range
type: knowledge_context
created_at: 2026-02-09T05:43:18.604297+00:00
updated_at: 2026-02-09T05:43:18.604297+00:00
iteration: 1
complexity: medium
stage: knowledge
scanned_categories:
  - grid
  - changelogs
  - index
---

# Knowledge Context

## Relevant Documents

- **grid/formula-syntax.md**
  - summary: Formula syntax guide covering SUM, AVERAGE, COUNT, MIN, MAX and range references (A1:B10). Relevant for status bar aggregation of selected ranges.
  - relevant sections: Functions - Mathematical (SUM, AVERAGE, COUNT, MIN, MAX), Basic Syntax - Ranges

## Patterns

- **Canvas-based rendering** (source: GridRenderer.ts)
  - Grid uses HTML Canvas for rendering. Selection highlight must be drawn via canvas 2D API, not DOM elements.
- **WASM bridge pattern** (source: src/core/RusheetAPI.ts)
  - All grid data access goes through WasmBridge. Selection state may need to be synced between TS and WASM layers.
- **Rust Selection model** (source: crates/cclab-grid-core/src/state/selection.rs)
  - Full Selection/SelectionRange/CellPosition structs exist in Rust with extend_to, add_range, multi-selection. Frontend needs to wire these up.

## Pitfalls

- Selection rendering must not conflict with cell border rendering from grid-styling-spec
- Large range selections (100k+ cells) could cause performance issues during highlight rendering - need to only render visible cells
- Mouse drag selection needs proper cleanup of mousemove/mouseup listeners to avoid memory leaks
