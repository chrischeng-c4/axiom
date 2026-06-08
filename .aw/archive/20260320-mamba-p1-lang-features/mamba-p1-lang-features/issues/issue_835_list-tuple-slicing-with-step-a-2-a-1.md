---
number: 835
title: "List/tuple slicing with step — a[::2], a[::-1]"
state: open
labels: [enhancement, P1, crate:mamba]
group: "slice-step"
---

# #835 — List/tuple slicing with step — a[::2], a[::-1]

## Summary

Support step parameter in slice operations:
```python
a = [1, 2, 3, 4, 5]
a[::2]    # [1, 3, 5]
a[::-1]   # [5, 4, 3, 2, 1]
a[1:4:2]  # [2, 4]
```

## Current State

Marked as TODO in `runtime/list_ops.rs`. Basic slicing (start:stop) works but step is not implemented.

## Scope

- **Runtime**: Implement step logic in list_ops, tuple_ops, string_ops
- **Codegen**: Ensure 3-argument slice is lowered correctly
- Negative step (reverse iteration) is the most important case
