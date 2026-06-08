---
change: grid-select-range
date: 2026-02-09
---

# Clarifications

## Q1: Feature Scope
- **Question**: Which select range features should be included?
- **Answer**: Full selection: drag-to-select, Shift+click extend, Shift+Arrow keys, range highlight rendering, and selection summary in status bar (e.g. SUM: 42)
- **Rationale**: Complete Excel/Sheets-like selection UX provides the best user experience and leverages existing Rust Selection data structures

## Q2: Multi-selection
- **Question**: Should Ctrl+Click multi-selection (non-contiguous ranges) be included?
- **Answer**: Yes, support Ctrl+Click to add additional disjoint ranges like Excel/Sheets
- **Rationale**: The Rust backend already supports multi-selection via Selection::add_range and additional ranges. Full multi-select support maximizes the value of existing backend code.

