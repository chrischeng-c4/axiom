---
change: context-menu
date: 2026-02-09
---

# Clarifications

## Q1: Feature Scope
- **Question**: Which context menu items should we include in the first version?
- **Answer**: Full set — Cut, Copy, Paste, Paste special, Insert rows/cols, Delete rows/cols, Create filter, Sort A→Z / Z→A, Conditional formatting, Data validation, Comment, More actions submenu. Matching Google Sheets context menu.
- **Rationale**: User wants feature parity with Google Sheets. The existing backend already has insert/delete rows/cols, filters, and sort APIs. Clipboard operations are new but essential.

## Q2: Rendering Approach
- **Question**: How should the context menu be rendered?
- **Answer**: DOM overlay — HTML/CSS menu positioned absolutely over the canvas element. Supports hover states, sub-menus, keyboard navigation, and standard CSS styling.
- **Rationale**: DOM overlay is the industry standard approach (used by Google Sheets, Excel Online). Easier to maintain, accessible, supports native text selection for labels.

## Q3: Git Workflow
- **Question**: Which git workflow for this change?
- **Answer**: in_place — work on current branch (tslibs)
- **Rationale**: Continuing on the same feature branch where grid-select-range was implemented. Context menu builds on top of selection features.

