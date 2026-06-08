---
number: 559
title: "Add targeted fixture: type parameter syntax (PEP 695)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #559 — Add targeted fixture: type parameter syntax (PEP 695)

## Context
PEP 695 (Python 3.12) introduced new type parameter syntax: `type X = int`, `def f[T]():`, `class C[T]:`.

## Test cases to cover
- Type alias: `type Point = tuple[float, float]`
- Generic function: `def f[T](x: T) -> T: ...`
- Generic class: `class Stack[T]: ...`
- Bounded TypeVar: `def f[T: int](): ...`
- Constrained TypeVar: `def f[T: (int, str)](): ...`
- ParamSpec: `def f[**P](): ...`
- TypeVarTuple: `def f[*Ts](): ...`
- Multiple params: `def f[T, U, *Ts, **P](): ...`
- Nested generics: `class A[T]: class B[U]: ...`
- Complex type aliases: `type Callback[**P, R] = Callable[P, R]`

## Task
Create `tests/fixtures/parse/edge_cases/type_params_pep695.py` with `# RUN: parse`.
