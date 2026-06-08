---
change: jet-remaining
group: bundle-optimization-hoisting
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: Variable Renaming
- **Answer**: According to #903, we should use module-prefixed names (e.g., _m0_foo, _m1_bar) for all top-level variables to avoid collisions when merging bodies into the unified scope.

### Q2: Scope Heuristics
- **Answer**: Yes, #903 explicitly states that we should skip flattening for modules that use eval(), with, or arguments to ensure scope safety and avoid side-effect complications.

### Q3: Mangler Integration
- **Answer**: The mangler will perform a single-pass minification on the concatenated/flattened output, which allows for better variable mangling with full visibility across the unified scope.

