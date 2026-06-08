---
change: grid-ui-toolbar
date: 2026-02-10
---

# Clarifications

## Q1: Toolbar Scope
- **Question**: Toolbar buttons should be visual only or should actions actually modify cell formatting in WASM?
- **Answer**: Full functionality - all toolbar buttons should be wired to actual cell formatting in WASM
- **Rationale**: User wants a complete working toolbar, not just a visual mockup

## Q2: Menu Bar
- **Question**: Should menu bar dropdowns actually open with items, or just show labels?
- **Answer**: Dropdowns with items - clicking menu labels opens dropdown menus with relevant items
- **Rationale**: User wants a fully interactive menu bar matching Google Sheets behavior

## Q3: Git Workflow
- **Question**: Which git workflow to use?
- **Answer**: in_place - work on current branch (tslibs)
- **Rationale**: Continue development on the existing tslibs branch

