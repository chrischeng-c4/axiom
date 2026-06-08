---
change: mamba-all-p1
group: data-structures
date: 2026-03-19
---

# Requirements

Implement complete data structure operations matching CPython 3.12:
- #835 Slicing with step: `a[::2]`, `a[::-1]`, `a[1:4:2]` for list, tuple, and string — implement step logic in list_ops.rs, tuple_ops.rs, string_ops.rs; ensure 3-argument slice is lowered correctly in codegen; negative step (reverse iteration) is highest priority
- #759 Data structure conformance: list (all methods, slicing, comprehension edge cases, comparisons), dict (all methods, PEP 584 merge `d1|d2`, comprehension, insertion-order, `__missing__`), set (all methods, set comprehension, frozenset), tuple (immutability, unpacking, hashing, comparison), str (47+ methods, f-string edge cases, format spec mini-language, Unicode normalization), bytes/bytearray (decode, hex, fromhex, slice, bytearray mutability)
Acceptance: all CPython 3.12 data structure conformance test cases pass; step slicing matches CPython behavior including edge cases (step=0 raises ValueError, negative indices).
