---
change_id: grid-merge-cells
type: codebase_context
created_at: 2026-02-10T03:38:50.159012+00:00
updated_at: 2026-02-10T03:38:50.159012+00:00
iteration: 1
complexity: high
stage: codebase
prism_tools_used:
  - grep
  - glob
  - read
---

# Codebase Context

## Analyzed Files

- **crates/cclab-grid-core/src/sheet.rs** — Core merge logic: merge_cells, unmerge_cells, get_merge_at, is_merged_slave, get_master_cell. Row/col insert/delete do NOT update merged_ranges (critical bug)
  - symbols: `merge_cells`, `unmerge_cells`, `get_merge_at`, `is_merged_slave`, `get_master_cell`, `get_merged_ranges`, `would_overlap_merge`, `insert_rows`, `delete_rows`, `insert_cols`, `delete_cols`
- **crates/cclab-grid-wasm/src/api.rs** — WASM exports: mergeCells, unmergeCells, getMergedRanges, isMergedSlave, getMergeInfo. MergeRangeData and MergeInfo types.
  - symbols: `mergeCells`, `unmergeCells`, `getMergedRanges`, `isMergedSlave`, `getMergeInfo`, `MergeRangeData`, `MergeInfo`
- **src/core/WasmBridge.ts** — TS bridge: mergeCells, unmergeCells, getMergedRanges, isMergedSlave, getMergeInfo - all complete
  - symbols: `mergeCells`, `unmergeCells`, `getMergedRanges`, `isMergedSlave`, `getMergeInfo`, `MergeInfo`, `MergeRange`
- **src/core/RusheetAPI.ts** — Public API: mergeCells, unmergeCells with event emission. onMergeCells/onUnmergeCells subscriptions.
  - symbols: `mergeCells`, `unmergeCells`, `getMergedRanges`, `isMergedSlave`, `getMergeInfo`, `onMergeCells`, `onUnmergeCells`
- **src/canvas/GridRenderer.ts** — Merge-aware rendering: getMergedWidth/Height, skips slave cells, renders merged cell spans. Complete.
  - symbols: `getMergedWidth`, `getMergedHeight`, `renderCells`, `renderSelection`
- **src/ui/Toolbar.ts** — Has merge button (⧉) that calls mergeCells. No unmerge. No toggle state.
  - symbols: `buildToolbar`, `merge button`
- **src/ui/InputController.ts** — Mouse/keyboard input. Need to verify merged cell selection and navigation handling.
  - symbols: `handleMouseDown`, `handleKeyDown`, `handleArrowKey`
- **src/ui/ContextMenu.ts** — Context menu component. No merge/unmerge items currently.
  - symbols: `ContextMenu`, `show`, `buildMenuItems`
- **src/types/events.ts** — MergeCellsEvent and UnmergeCellsEvent types defined. Complete.
  - symbols: `MergeCellsEvent`, `UnmergeCellsEvent`

## Dependency Graph

- sheet.rs (merge logic) -> api.rs (WASM exports) -> WasmBridge.ts -> RusheetAPI.ts -> UI components
- GridRenderer.ts reads merge info from WasmBridge for rendering
- InputController.ts needs merge info for navigation/selection
- Toolbar.ts and ContextMenu.ts call RusheetAPI for merge/unmerge actions
