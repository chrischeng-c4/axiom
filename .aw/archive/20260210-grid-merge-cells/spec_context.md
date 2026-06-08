---
change_id: grid-merge-cells
type: spec_context
created_at: 2026-02-10T03:38:35.536063+00:00
updated_at: 2026-02-10T03:38:35.536063+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-grid
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **grid-styling-spec** (group: cclab-grid)
  - relevance: high
  - reason: Defines CellFormat including merge-related styling properties
  - key sections: CellFormat structure, Merge rendering
- **selection-wasm-api** (group: cclab-grid)
  - relevance: high
  - reason: Selection API must handle merged regions - selecting a slave cell should select the whole merge
  - key sections: Selection model, Active cell tracking
- **selection-ui-interaction** (group: cclab-grid)
  - relevance: high
  - reason: Keyboard/mouse interaction must account for merged cells during navigation
  - key sections: Arrow key navigation, Click handling
- **selection-rendering** (group: cclab-grid)
  - relevance: medium
  - reason: Selection highlight must span merged cell dimensions
  - key sections: Selection highlight rendering
- **toolbar-formatting** (group: cclab-grid)
  - relevance: medium
  - reason: Toolbar has merge button that needs toggle behavior for merge/unmerge
  - key sections: Merge button, Button state sync
- **context-menu-operations** (group: cclab-grid)
  - relevance: medium
  - reason: Context menu needs merge/unmerge items
  - key sections: Menu items, Action wiring
- **context-menu-ui** (group: cclab-grid)
  - relevance: low
  - reason: Context menu UI pattern for adding merge/unmerge items
  - key sections: Menu rendering

## Dependencies

- selection-wasm-api depends on grid-styling-spec for CellFormat
- selection-ui-interaction depends on selection-wasm-api
- toolbar-formatting depends on selection-wasm-api for merge state
- context-menu-operations depends on selection-wasm-api for merge detection

## Gaps

- No existing spec for merge cell behavior specifically
- Selection specs don't mention merged cell navigation
- No spec for row/col insert/delete adjusting merged ranges
