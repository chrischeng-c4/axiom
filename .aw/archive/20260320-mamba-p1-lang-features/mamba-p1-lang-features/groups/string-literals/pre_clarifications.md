---
change: mamba-p1-lang-features
group: string-literals
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Bytes runtime representation
- **Answer**: bytes as NaN-boxed heap object wrapping Vec<u8>, ObjKind::Bytes in existing value model.

### Q2: General
- **Question**: f-string scope
- **Answer**: f-strings not in scope for this change unless lexer prefix handling requires it.

### Q3: General
- **Question**: \N{name} Unicode named escapes
- **Answer**: Full \N{name} Unicode named escapes — embed or query Unicode name database.

### Q4: General
- **Question**: Bytes operations scope
- **Answer**: Full CPython bytes API: find, replace, decode, startswith, split, join, hex, etc.

