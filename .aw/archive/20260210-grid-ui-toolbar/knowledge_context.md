---
change_id: grid-ui-toolbar
type: knowledge_context
created_at: 2026-02-10T02:45:23.511696+00:00
updated_at: 2026-02-10T02:45:23.511696+00:00
iteration: 1
complexity: high
stage: knowledge
scanned_categories:
  - grid
  - 30-claude
  - 40-mcp
  - changelogs
---

# Knowledge Context

## Relevant Documents

- **cclab/knowledge/grid/index.md**
  - summary: Grid knowledge index - no toolbar-specific content found
- **cclab/knowledge/index.md**
  - summary: Knowledge base index listing all categories
- **cclab/knowledge/30-claude/skills.md**
  - summary: Claude skills documentation - no UI toolbar patterns
- **cclab/knowledge/40-mcp/claude-mcp.md**
  - summary: MCP integration docs - not directly relevant to toolbar UI

## Patterns

- **React functional component with forwardRef** (source: src/react/RuSheet.tsx)
  - RuSheet uses forwardRef + useImperativeHandle to expose grid API to parent. New toolbar should be a child component within this layout.
- **Event-driven UI updates via rusheet.onSelectionChange** (source: src/core/RusheetAPI.ts)
  - Status bar and formula bar listen to onSelectionChange for reactive updates. Toolbar must follow same pattern to sync formatting state.
- **DOM overlay pattern for UI components** (source: src/ui/ContextMenu.ts)
  - ContextMenu and FilterDropdown use position:fixed DOM elements appended to body. Menu bar dropdowns should follow this pattern.
- **CSS class conventions for toolbar** (source: src/styles/main.css)
  - Existing CSS defines .toolbar, .toolbar-btn, .toolbar-separator classes with #f5f5f5 background and grid-line border colors.
- **WASM-mediated formatting via RusheetAPI** (source: src/core/RusheetAPI.ts)
  - setRangeFormat(), mergeCells(), unmergeCells(), undo(), redo() are already exposed through rusheet singleton.

## Pitfalls

- No external icon library in package.json - must use inline SVGs or text labels for toolbar icons
- Performance: avoid frequent WASM calls during drag selection; use 100ms debounce pattern from aggregation logic
- State sync: when multi-cell range has mixed formats, toolbar should show indeterminate state or reflect active cell only
- CellFormat supports bold/italic/underline/fontSize/textColor/backgroundColor/alignment but no strikethrough yet in WASM
