---
change: mamba-p1-lang-features
group: slice-step
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Runtime dispatch
- **Answer**: Follow existing slice dispatch pattern for step implementation.

### Q2: General
- **Question**: 3-arg slice in AST
- **Answer**: Add parser/codegen changes as needed to pass step argument through to runtime.

### Q3: General
- **Question**: String slicing codepoints vs bytes
- **Answer**: String step-slicing on Unicode codepoints, matching CPython.

### Q4: General
- **Question**: Zero step error handling
- **Answer**: a[::0] raises Python ValueError at runtime, matching CPython.

