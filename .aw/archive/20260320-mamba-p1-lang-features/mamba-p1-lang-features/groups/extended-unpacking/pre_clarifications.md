---
change: mamba-p1-lang-features
group: extended-unpacking
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Starred variable type
- **Answer**: Starred variable typed as list[T], T inferred from RHS element type, matching Py3.12.

### Q2: General
- **Question**: Star in function calls
- **Answer**: Splat in function calls (*args, **kwargs) also addressed — match Py3.12 behavior.

### Q3: General
- **Question**: Nested unpacking with star
- **Answer**: Include nested destructuring with stars (e.g., a, (*b, c) = ...) — full Py3.12 behavior.

### Q4: General
- **Question**: Codegen runtime helper
- **Answer**: Runtime helper: runtime::unpack_star(iter, n_before, n_after) for cleaner codegen.

