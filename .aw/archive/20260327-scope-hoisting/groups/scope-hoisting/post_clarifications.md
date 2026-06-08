---
change: scope-hoisting
group: scope-hoisting
date: 2026-03-26
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
Jet AOT build produces ~206.8 KB vs Vite's ~192 KB (~7.7% larger) due to per-module wrapper overhead. Scope hoisting inlines single-importer modules to reduce wrappers and enable cross-module DCE.

### Success Criteria
(not provided)

### Boundary
(not provided)

