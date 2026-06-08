---
number: 846
title: "Global and nonlocal statements"
state: open
labels: [enhancement, P1, crate:mamba]
group: "syntax-features"
---

# #846 — Global and nonlocal statements

## Summary

Support `global` and `nonlocal` statements for variable scope control:
```python
x = 0
def inc():
    global x
    x += 1

def outer():
    count = 0
    def inner():
        nonlocal count
        count += 1
    inner()
    return count  # 1
```

## Scope

- **Parser**: Parse `global` and `nonlocal` statements
- **Resolver**: Mark variables as global/nonlocal in scope chain
- **Codegen**: Emit appropriate load/store to outer scope (closure capture or global table)
