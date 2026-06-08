---
number: 830
title: "PEP 695 type parameter syntax — full generics support"
state: open
labels: [enhancement, P1, crate:mamba]
group: "syntax-features"
---

# #830 — PEP 695 type parameter syntax — full generics support

## Summary

Complete PEP 695 type parameter syntax support. Currently partially implemented.

## Missing pieces

- Type parameter bounds: `def f[T: int](x: T) -> T`
- `ParamSpec`: `def f[**P](fn: Callable[P, int]) -> Callable[P, str]`
- `TypeVarTuple`: `def f[*Ts](*args: *Ts) -> tuple[*Ts]`
- Generic class keywords: `class MyDict[K, V](dict[K, V])`
- Type alias statement: `type Vector = list[float]`

## References

- [PEP 695 – Type Parameter Syntax](https://peps.python.org/pep-0695/)
