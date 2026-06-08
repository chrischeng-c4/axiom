---
change: grid-merge-cells
date: 2026-02-10
---

# Clarifications

## Q1: Scope
- **Question**: What scope of merge/unmerge work do you want? Core merge logic, WASM API, rendering, and events already exist. Main gaps: (1) row/col insert/delete don't update merged ranges, (2) no unmerge button/UX, (3) no merge in context menu, (4) sort doesn't handle merges.
- **Answer**: Full + selection: All fixes plus keyboard navigation through merged cells and selection expansion to cover merged regions
- **Rationale**: User wants comprehensive Google Sheets-aligned merge behavior including the critical row/col shift bug fix, UX improvements, and proper selection/navigation handling for merged cells

## Q2: Git workflow
- **Question**: Which git workflow should be used for this change?
- **Answer**: in_place: Work on current tslibs branch directly
- **Rationale**: Consistent with previous changes on this branch

