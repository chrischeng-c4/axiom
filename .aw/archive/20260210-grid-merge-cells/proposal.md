---
id: grid-merge-cells
type: proposal
version: 1
created_at: 2026-02-10T03:42:05.648947+00:00
updated_at: 2026-02-10T03:42:05.648947+00:00
author: mcp
status: proposed
iteration: 1
summary: "Fix merge cell gaps: row/col shift updates, toggle UX, selection navigation, context menu"
history:
  - timestamp: 2026-02-10T03:42:05.648947+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
impact:
  scope: minor
  affected_files: 5
  new_files: 0
affected_specs:
  - id: merge-row-col-shift
    path: specs/merge-row-col-shift.md
    depends: []
  - id: merge-selection-navigation
    path: specs/merge-selection-navigation.md
    depends: [merge-row-col-shift]
  - id: merge-ui-controls
    path: specs/merge-ui-controls.md
    depends: [merge-selection-navigation]
---

<proposal>

# Change: grid-merge-cells

## Summary

Fix merge cell gaps: row/col shift updates, toggle UX, selection navigation, context menu

## Why

The cell merge infrastructure (Rust core, WASM API, TS bridge, renderer) is substantially complete, but several critical gaps prevent production-quality Google Sheets-aligned behavior.

The most critical bug is that row/column insert/delete operations in sheet.rs do not adjust the merged_ranges vector. This means inserting a row inside a merged region silently corrupts the merge coordinates, leading to rendering artifacts and data loss.

On the UX side, the toolbar only has a merge button with no unmerge capability and no state toggle. Google Sheets uses a single merge button that toggles: if the selection is already merged, clicking it unmerges. There's also no merge/unmerge option in the right-click context menu. Additionally, keyboard navigation doesn't handle merged cells — arrow keys should skip slave cells and jump across merged regions, and clicking a slave cell should select the entire merge region.

This change addresses all these gaps to bring merge/unmerge functionality to full Google Sheets parity.

## What Changes

- Fix row/col insert/delete to adjust merged_ranges coordinates in sheet.rs (shift, expand, shrink, or remove merges as needed)
- Add merge-aware keyboard navigation in InputController — arrow keys skip slave cells, Tab/Enter respect merge boundaries
- Make clicking a slave cell select the entire merged region (expand selection to merge bounds)
- Toggle merge button in Toolbar — detect if selection is merged and unmerge, or merge if not; show active state
- Add Merge cells / Unmerge cells items to the right-click context menu with appropriate enable/disable logic
- Add sort protection: block or warn when sorting ranges that overlap merged regions

## Impact

- **Scope**: minor
- **Affected Files**: ~5
- **New Files**: ~0
- Affected specs:
  - `merge-row-col-shift` (no dependencies)
  - `merge-selection-navigation` → depends on: `merge-row-col-shift`
  - `merge-ui-controls` → depends on: `merge-selection-navigation`
- Affected code: `crates/cclab-grid-core/src/sheet.rs`, `src/ui/InputController.ts`, `src/ui/Toolbar.ts`, `src/ui/ContextMenu.ts`, `src/canvas/GridRenderer.ts`

</proposal>
