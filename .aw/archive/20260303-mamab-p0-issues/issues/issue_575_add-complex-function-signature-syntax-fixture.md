---
number: 575
title: "Add complex function signature syntax fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #575 — Add complex function signature syntax fixture

## Context
Python function signatures are among the most syntactically complex constructs in the language.

## Test cases
```python
# All parameter kinds in order (PEP 570)
def f(pos_only, /, normal, *, kw_only): pass
def g(p1, p2, /, n1, n2, *, k1, k2): pass
def h(p, /, n, *args, k, **kwargs): pass

# Complex defaults
def f(x=[], y={}, z=()): pass
def g(x=lambda: 1, y=None): pass
def h(x=1+2, y=f()): pass

# Type annotations
def f(x: int, y: str = "hello") -> bool: pass
def g(x: list[int], y: dict[str, Any]) -> Optional[int]: pass
def h(x: int | str | None) -> tuple[int, ...]: pass

# Complex annotations
def f(x: 'forward_ref') -> 'ReturnType': pass
def g(x: Annotated[int, "positive"]) -> None: pass
def h(callback: Callable[[int, str], bool]) -> None: pass

# Generic functions (PEP 695)
def first[T](items: list[T]) -> T: pass
def pair[T, U](a: T, b: U) -> tuple[T, U]: pass
def variadic[*Ts](*args: *Ts) -> tuple[*Ts]: pass
def paramspec[**P](f: Callable[P, int]) -> Callable[P, str]: pass

# Async variants
async def f(x: int) -> str: pass
async def g(*args, **kwargs): pass

# Decorated
@decorator
@another(arg)
def f(): pass

# Very long signature
def very_long_function(
    first_param: int,
    second_param: str,
    third_param: float = 0.0,
    /,
    normal_param: bool = True,
    *args: tuple[int, ...],
    keyword_only: str = "default",
    another_kw: Optional[int] = None,
    **kwargs: Any,
) -> dict[str, Any]:
    pass

# Nested def
def outer(x):
    def inner(y):
        def innermost(z):
            return x + y + z
        return innermost
    return inner
```

## Task
Create `tests/fixtures/parse/edge_cases/function_signatures.py` with `# RUN: parse`.
