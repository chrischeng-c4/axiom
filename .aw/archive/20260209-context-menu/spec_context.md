---
change_id: context-menu
type: spec_context
created_at: 2026-02-09T08:23:10.711170+00:00
updated_at: 2026-02-09T08:23:10.711170+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-grid
  - cclab-grid-db
  - cclab-core
  - cclab-genesis
  - cclab-aurora
  - cclab-cli
  - cclab-ion
  - cclab-meteor
  - cclab-nebula
  - cclab-nova
  - cclab-nucleus
  - cclab-orbit
  - cclab-photon
  - cclab-prism
  - cclab-probe
  - cclab-pulsar-array-core
  - cclab-quasar
  - cclab-server
  - cclab-shield
  - cclab-titan
  - genesis
  - nebula
---

# Spec Context

## Relevant Specs

- **selection-ui-interaction** (group: cclab-grid)
  - relevance: high
  - reason: Defines mouse/keyboard interaction patterns. Context menu right-click integrates with the selection state machine (Idle state triggers context menu on right-click). Selection state determines which menu items are enabled.
  - key sections: R1 - Drag-to-select state machine, R5 - Click resets to single cell, Flow Diagram
- **grid-styling-spec** (group: cclab-grid)
  - relevance: medium
  - reason: Context menu includes conditional formatting option. CellFormat model defines applicable styles.
  - key sections: R1 - Cell Borders, R2 - Pattern Fills, API Specification
- **grid-io-spec** (group: cclab-grid)
  - relevance: low
  - reason: Clipboard paste-special may need format-aware parsing (CSV, plain text). Export formats inform paste behavior.
  - key sections: R1 - XLSX Support, R2 - CSV Support
- **grid-formula-functions-spec** (group: cclab-grid)
  - relevance: low
  - reason: Sort operations in context menu interact with formula references. Paste operations may involve formula adjustment.
  - key sections: R1 - INDEX Function
- **grid-performance-spec** (group: cclab-grid)
  - relevance: low
  - reason: Insert/delete row/col operations on large datasets need to consider performance. Formula recalculation after structural changes.
  - key sections: R2 - 100k+ Row Support

## Dependencies

- context-menu depends on selection-ui-interaction: right-click context is determined by current selection state
- context-menu depends on grid-styling-spec: conditional formatting menu item uses CellFormat model
- context-menu depends on grid-io-spec: clipboard operations involve format detection

## Gaps

- No spec for context menu UI component (positioning, styling, keyboard nav, sub-menus)
- No spec for clipboard operations (Cut/Copy/Paste/Paste Special) in the grid
- No spec for insert/delete row/col WASM API — these operations exist in Rust core but no formal spec
- No spec for sort operations (Sort A-Z, Sort Z-A) accessible from UI
- No spec for data validation UI or conditional formatting dialogs
