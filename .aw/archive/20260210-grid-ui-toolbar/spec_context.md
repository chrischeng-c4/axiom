---
change_id: grid-ui-toolbar
type: spec_context
created_at: 2026-02-10T02:41:59.499729+00:00
updated_at: 2026-02-10T02:41:59.499729+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-grid
  - cclab-core
---

# Spec Context

## Relevant Specs

- **selection-wasm-api** (group: cclab-grid)
  - relevance: high
  - reason: Provides selection data and aggregation for toolbar display
  - key sections: R2 - WASM selection getter, R5 - WASM selection aggregation
- **selection-ui-interaction** (group: cclab-grid)
  - relevance: high
  - reason: Source of selection events to refresh toolbar state
  - key sections: R1 - Drag-to-select, Acceptance Criteria
- **grid-styling-spec** (group: cclab-grid)
  - relevance: high
  - reason: Defines the CellFormat data model for all styling operations
  - key sections: Flow Diagram (CellFormat), API Specification
- **selection-status-bar** (group: cclab-grid)
  - relevance: medium
  - reason: Reference for selection-reactive UI components
  - key sections: R2 - Aggregation on selection change
- **context-menu-ui** (group: cclab-grid)
  - relevance: medium
  - reason: Provides patterns for dynamic action generation based on selection context
  - key sections: DOM overlay pattern, MenuItem interface
- **selection-rendering** (group: cclab-grid)
  - relevance: low
  - reason: Canvas rendering details for header gridlines portion of the change
  - key sections: renderHeaders, gridToScreen

## Dependencies

- grid-ui-toolbar depends on selection-wasm-api for current range data
- grid-ui-toolbar depends on selection-ui-interaction for SelectionChangeEvent
- grid-ui-toolbar outputs data matching grid-styling-spec CellFormat schema
- grid-ui-toolbar follows context-menu-ui DOM overlay pattern

## Gaps

- No styling-wasm-api to apply formatting to ranges (setSelectionFormat)
- No toolbar component spec (layout, button groupings, component hierarchy)
- No toolbar state management spec (syncing selection with button states)
- No menu bar dropdown spec (File/Edit/View/Insert/Format/Data menus)
- No header gridlines rendering spec (separators between column/row headers)
