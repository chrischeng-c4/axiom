---
change: mamba-p1-lang-features
group: scope-modifiers
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Closure/upvalue representation
- **Answer**: Integrate nonlocal with existing closure/capture mechanism in codegen.

### Q2: General
- **Question**: Undeclared global variable
- **Answer**: Silently allowed — defer to runtime NameError if accessed before assignment, matching CPython.

### Q3: General
- **Question**: Type checker integration
- **Answer**: Type checker updated to respect global/nonlocal annotations for cross-scope type inference.

### Q4: General
- **Question**: Interaction with closures already captured
- **Answer**: nonlocal upgrades implicit read-only capture to mutable capture, matching CPython.

