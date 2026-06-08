---
number: 383
title: "feat(mamba): super() runtime implementation"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #383 — feat(mamba): super() runtime implementation

## Summary
Implement Python `super()` at runtime so method calls chain through the MRO correctly.

## Required
- Zero-arg `super()` — automatically resolve class and instance from call frame
- `super(Type, obj)` — explicit form
- `super().method(args)` — dispatch to next class in MRO
- `super().__init__(args)` — parent class initialization
- Multiple inheritance: `super()` follows C3 MRO, not just direct parent

## Implementation Notes
- `class.rs` has MRO (C3) computation but `super()` is not wired
- Codegen needs to pass implicit `__class__` cell variable to methods
- `super()` object needs to be a special MbObject that proxies attribute lookup starting from the next MRO entry
