---
number: 831
title: "Dict literal unpacking — {**d1, **d2, key: val} syntax"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "parser-syntax"
---

# #831 — Dict literal unpacking — {**d1, **d2, key: val} syntax

## Summary

Support dict unpacking in dict literals (PEP 448):
```python
base = {"a": 1, "b": 2}
extended = {**base, "c": 3}
merged = {**d1, **d2}
```

Currently listed in `cpython_known_failures.toml` as unsupported.

## Scope

- **Parser**: Handle `**expr` in dict literal expressions
- **Type checker**: Infer resulting dict type from spread operands
- **Codegen**: Emit dict creation + update calls

## References

- [PEP 448 – Additional Unpacking Generalizations](https://peps.python.org/pep-0448/)
