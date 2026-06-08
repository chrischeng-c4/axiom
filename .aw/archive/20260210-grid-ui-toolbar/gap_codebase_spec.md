---
change_id: grid-ui-toolbar
type: gap_codebase_spec
created_at: 2026-02-10T02:46:20.791117+00:00
updated_at: 2026-02-10T02:46:20.791117+00:00
---

# Gap Analysis: Codebase vs Spec

## Code Without Matching Spec (HIGH)

### 1. Header Gridline Separators [HIGH]
- **Code**: `src/canvas/GridRenderer.ts:renderHeaders()` (lines 481-560)
- **Gap**: Code draws header backgrounds and text but only 2 border lines. No spec exists for individual column/row header separator gridlines.
- **Impact**: User reported missing separator lines between column headers and row headers.

### 2. Toolbar UI Component [HIGH]
- **Code**: `index.html` has basic `.toolbar` div with Save/Load/Export buttons
- **Gap**: No spec for a Google Sheets-style toolbar with formatting buttons (bold/italic/underline/colors/borders/merge/alignment), undo/redo, font selector, font size.
- **Impact**: Core deliverable of this change.

### 3. Menu Bar with Dropdowns [HIGH]
- **Code**: No menu bar exists in current codebase
- **Gap**: No spec for File/Edit/View/Insert/Format/Data menu bar with dropdown menus containing actionable items.
- **Impact**: Core deliverable of this change.

### 4. Formula Bar Redesign [MEDIUM]
- **Code**: `index.html` has basic formula bar with cell-address span and text input
- **Gap**: No spec for Google Sheets-style formula bar with cell address dropdown, fx icon, and styled input.
- **Impact**: Part of the toolbar redesign scope.

## Specs Without Matching Implementation (for this change)

### 5. grid-styling-spec CellFormat Application [MEDIUM]
- **Spec**: `cclab-grid/grid-styling-spec` defines CellFormat with bold/italic/underline/fontSize/textColor/backgroundColor/alignment
- **Gap**: WASM API `setRangeFormat` exists but no toolbar UI to invoke it. Format reading via `getCellFormat` exists but no toolbar state sync.
- **Impact**: Toolbar buttons need to wire into existing WASM formatting API.

### 6. selection-wasm-api Aggregation Display [LOW]
- **Spec**: `cclab-grid/selection-wasm-api` R5 defines selection aggregation
- **Gap**: Status bar shows aggregation but toolbar doesn't show selection-related formatting info.
- **Impact**: Toolbar should reflect active cell's current format state."