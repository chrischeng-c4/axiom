---
change_id: context-menu
type: gap_codebase_knowledge
created_at: 2026-02-09T08:27:58.027069+00:00
updated_at: 2026-02-09T08:27:58.027069+00:00
---

# Gap Analysis: Codebase vs Knowledge

## Convention Violations

### MEDIUM severity
- **No knowledge doc for UI component patterns** — The codebase has established DOM overlay patterns (FilterDropdown.ts, CellEditor.ts) but these patterns are not documented in `cclab/knowledge/`. New contributors may not discover these conventions.
  - Files: `src/ui/FilterDropdown.ts`, `src/ui/CellEditor.ts`
  - Knowledge gap: `cclab/knowledge/grid/` only has `formula-syntax.md`

### LOW severity
- **Event emitter pattern undocumented** — RusheetAPI's convention of wrapping WasmBridge calls with event emission is not in any knowledge doc. Only learned from codebase exploration.
  - Files: `src/core/RusheetAPI.ts`
  - Knowledge gap: No `cclab/knowledge/grid/architecture.md` or similar

## Pattern Mismatches

### MEDIUM severity
- **Clipboard API not in knowledge base** — The knowledge base has no guidance on browser API usage (navigator.clipboard, Clipboard API permissions, secure context requirements). The context menu clipboard feature will be the first browser API integration beyond basic DOM/Canvas.
  - Impact: Developers may not know about secure context requirement or permission prompts

### LOW severity
- **Sort behavior undocumented** — `formula-syntax.md` documents formula syntax but says nothing about sort behavior with formulas (do formula references update? do sorted ranges maintain formula integrity?).
  - Files: `cclab/knowledge/grid/formula-syntax.md`

## No Gaps Found
- InputController event handling patterns align with knowledge conventions
- Z-index layering follows established codebase patterns (CellEditor z-index: 1000)"