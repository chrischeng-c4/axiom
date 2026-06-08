---
change: lens-comprehensive
group: refactoring-engine
date: 2026-03-12
status: answered
---

# Pre-Clarifications

### Q1: Refactoring scope
- **Answer**: Implement all 6 operations: Rename, Extract Function, Extract Variable, Inline, Move Definition, Change Signature.

### Q2: Cross-file rename
- **Answer**: Cross-file rename using project-wide index via existing SymbolTable + daemon.

### Q3: Undo support
- **Answer**: No undo. One-way apply. Users can use git revert.

