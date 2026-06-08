---
change_id: context-menu
type: gap_codebase_spec
created_at: 2026-02-09T08:27:29.216595+00:00
updated_at: 2026-02-09T08:27:29.216595+00:00
---

# Gap Analysis: Codebase vs Specs

## Code Exists, No Spec

### HIGH severity
- **Insert/delete rows/cols** — `RusheetAPI.insertRows()`, `deleteRows()`, `insertCols()`, `deleteCols()` are fully implemented in WASM and TS bridge but have NO formal spec. Context menu will expose these operations to users.
  - Files: `src/core/RusheetAPI.ts` (L749-767), `crates/cclab-grid-wasm/src/api.rs` (L629-680)

- **Sort range** — `RusheetAPI.sortRange()` is implemented but has no spec for UI-triggered sort behavior (ascending/descending, sort by which column, header detection).
  - Files: `src/core/RusheetAPI.ts` (L773), `crates/cclab-grid-wasm/src/api.rs` (L697)

### MEDIUM severity
- **Cell merge/unmerge** — `RusheetAPI.mergeCells()`, `unmergeCells()` exist but no spec defines when merge is valid (e.g., can't merge across filtered rows).
  - Files: `src/core/RusheetAPI.ts` (L792-809)

- **Data validation** — `api.rs` has `addDataValidation()` but no UI spec for how users configure validation rules via context menu.
  - Files: `crates/cclab-grid-wasm/src/api.rs` (L1059-1127)

- **Conditional formatting** — `api.rs` has `addConditionalFormatting()` but no UI spec for rule creation dialog.
  - Files: `crates/cclab-grid-wasm/src/api.rs` (L962-993)

### LOW severity
- **Filter operations** — `FilterDropdown.ts` and `RusheetAPI.applyColumnFilter()` exist. Context menu 'Create filter' item just needs to trigger existing filter UI.
  - Files: `src/ui/FilterDropdown.ts`, `src/core/RusheetAPI.ts` (L850-859)

## Specs Exist, No Implementation

### HIGH severity
- **No context menu spec** — No spec exists for the context menu UI component itself (DOM structure, positioning algorithm, keyboard navigation, sub-menu behavior, dynamic item enablement based on selection state).

- **No clipboard spec** — No spec and no implementation for Cut/Copy/Paste/Paste Special. The `navigator.clipboard` API integration, TSV/HTML serialization, and formula reference adjustment on paste are all missing.
  - Impact: 4 of the top menu items (Cut, Copy, Paste, Paste special) have zero backend or frontend support.

### MEDIUM severity
- **No comment spec** — No spec or implementation for cell comments/notes. This is a context menu item in Google Sheets but would require a new data model.

- **No spec for right-click interaction model** — `selection-ui-interaction` spec covers left-click and keyboard but does NOT define right-click behavior (should right-click change selection? Should it select the clicked cell if outside current selection?).

### LOW severity
- **No 'More actions' submenu spec** — Google Sheets has a 'More actions' submenu with additional operations. Scope unclear."