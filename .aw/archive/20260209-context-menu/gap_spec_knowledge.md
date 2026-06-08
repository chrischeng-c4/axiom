---
change_id: context-menu
type: gap_spec_knowledge
created_at: 2026-02-09T08:28:19.853639+00:00
updated_at: 2026-02-09T08:28:19.853639+00:00
---

# Gap Analysis: Specs vs Knowledge

## Spec Responsibilities Contradicting Knowledge Architecture

- None found. The existing grid specs (styling, I/O, formula, performance) do not contradict knowledge base conventions.

## Knowledge Patterns Not Reflected in Specs

### MEDIUM severity
- **Event emitter pattern** — Knowledge context identified the RusheetAPI event emitter pattern as a core convention, but no spec documents this pattern or requires context menu operations to follow it. The `selection-ui-interaction` spec mentions 'route through RusheetAPI' implicitly but doesn't formalize the pattern.
  - Knowledge source: codebase convention (RusheetAPI.ts)
  - Spec gap: No architectural spec for UI operation routing

### LOW severity
- **DOM overlay pattern** — Knowledge context documented CellEditor and FilterDropdown DOM overlay patterns, but no spec formalizes the overlay positioning algorithm (boundary checking, z-index layering, parent container selection).
  - Knowledge source: codebase convention (FilterDropdown.ts, CellEditor.ts)
  - Spec gap: No UI component architecture spec

## Responsibility Boundary Misalignments

### MEDIUM severity
- **Clipboard responsibility unclear** — Grid specs cover I/O for file formats (XLSX, CSV) but don't address clipboard as an I/O channel. Should clipboard serialization live in grid-core (Rust), grid-wasm, or pure TypeScript? No spec or knowledge doc defines this boundary.
  - Related spec: `grid-io-spec` (file I/O only)
  - Impact: Context menu Cut/Copy/Paste implementation needs this decision

### LOW severity
- **Sort vs Filter responsibility** — Grid specs don't clarify the relationship between sort and filter. Can you sort filtered data? Does sort respect hidden rows? Knowledge base is silent on this.
  - Related spec: None (sort has no spec)
  - Impact: Context menu 'Sort A→Z' behavior on filtered datasets"