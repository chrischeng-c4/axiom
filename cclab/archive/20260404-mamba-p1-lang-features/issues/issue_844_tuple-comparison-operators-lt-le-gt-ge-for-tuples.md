---
number: 844
title: "Tuple comparison operators — lt, le, gt, ge for tuples"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "runtime-ops"
---

# #844 — Tuple comparison operators — lt, le, gt, ge for tuples

## Summary

Implement lexicographic comparison for tuples:
```python
(1, 2, 3) < (1, 2, 4)   # True
(1, 2) < (1, 2, 0)      # True (shorter is less)
(1, "a") < (1, "b")     # True
```

## Current State

Marked as TODO in `runtime/tuple_ops.rs`. Equality works but ordering operators are not implemented.

## Scope

- Element-by-element lexicographic comparison
- Handle tuples of different lengths
- Handle heterogeneous element types (follow CPython's type ordering)
