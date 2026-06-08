---
number: 845
title: "Star expressions — *a, b = [1, 2, 3] extended unpacking"
state: open
labels: [enhancement, P1, crate:mamba]
group: "extended-unpacking"
---

# #845 — Star expressions — *a, b = [1, 2, 3] extended unpacking

## Summary

Support extended iterable unpacking (PEP 3132):
```python
first, *rest = [1, 2, 3, 4]        # first=1, rest=[2, 3, 4]
first, *mid, last = [1, 2, 3, 4]   # first=1, mid=[2, 3], last=4
*init, last = [1, 2, 3]            # init=[1, 2], last=3
```

Also support star expressions in function calls and list/tuple literals:
```python
a = [1, 2]
b = [*a, 3, 4]        # [1, 2, 3, 4]
f(*args, **kwargs)     # already may work for calls
```

## Scope

- **Parser**: Star assignment targets in tuple/list unpacking
- **Type checker**: Infer list type for starred variable
- **Codegen**: Emit iterator split logic
