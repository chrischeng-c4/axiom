---
number: 832
title: "Parenthesized with statements (PEP 617) — multi-context managers"
state: open
labels: [enhancement, P1, crate:mamba]
group: "syntax-features"
---

# #832 — Parenthesized with statements (PEP 617) — multi-context managers

## Summary

Support parenthesized context managers introduced in Python 3.10:

```python
with (
    open("a.txt") as a,
    open("b.txt") as b,
):
    ...
```

Currently listed in `cpython_known_failures.toml` as unsupported.

## Scope

- **Parser**: Allow parenthesized `with` item list with trailing comma
- Rest of the pipeline (type checker, lowering, codegen) should work if single-`with` already does

## References

- [PEP 617 – New Parser](https://peps.python.org/pep-0617/)
