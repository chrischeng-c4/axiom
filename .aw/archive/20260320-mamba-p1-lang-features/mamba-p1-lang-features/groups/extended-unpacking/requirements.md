---
change: mamba-p1-lang-features
group: extended-unpacking
date: 2026-03-20
---

# Requirements

Support PEP 3132 extended iterable unpacking and star spread in constructors.

- Assignment unpacking: `first, *rest = [1, 2, 3, 4]`, `first, *mid, last = [1, 2, 3, 4]`, `*init, last = [1, 2, 3]`.
- Star spread in list/tuple literals: `b = [*a, 3, 4]` expands `a` inline.
- Parser: recognize star assignment targets (`*name`) in tuple/list unpacking on the left-hand side of assignments.
- Type checker: infer `list[T]` for the starred variable based on RHS element type; enforce at most one star target per unpacking (Python `SyntaxError` otherwise).
- Codegen: emit iterator split logic — consume first N elements, collect middle into a list, consume last M elements.
