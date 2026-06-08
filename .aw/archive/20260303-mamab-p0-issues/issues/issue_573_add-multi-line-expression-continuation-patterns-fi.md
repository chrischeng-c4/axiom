---
number: 573
title: "Add multi-line expression continuation patterns fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #573 — Add multi-line expression continuation patterns fixture

## Context
Python has implicit and explicit line continuation rules that affect parsing.

## Test cases
```python
# Implicit continuation in brackets
x = (
    1 + 2 +
    3 + 4
)

y = [
    1, 2,
    3, 4,
]

z = {
    'a': 1,
    'b': 2,
}

# Explicit backslash continuation
x = 1 + \
    2 + \
    3

if a and \
   b and \
   c:
    pass

# Mixed
result = (func(
    arg1,
    arg2,
) + other_func(
    arg3,
))

# Multi-line function def
def very_long_function_name(
    param1: int,
    param2: str,
    *args: Any,
    keyword_only: bool = False,
    **kwargs: Any,
) -> Optional[int]:
    pass

# Multi-line class
class MyClass(
    Base1,
    Base2,
    metaclass=Meta,
):
    pass

# Multi-line decorator
@decorator(
    arg1,
    arg2,
)
def f(): pass

# Multi-line if/while/for
if (
    condition1
    and condition2
    and condition3
):
    pass

# Multi-line with
with (
    open('a') as f,
    open('b') as g,
):
    pass

# Multi-line assert
assert (
    very_long_condition
), "error message"

# Multi-line return
return (
    value1,
    value2,
)

# Multi-line assignment
(
    a,
    b,
    c,
) = func()
```

## Task
Create `tests/fixtures/parse/edge_cases/multiline_continuation.py` with `# RUN: parse`.
