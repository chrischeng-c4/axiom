---
number: 570
title: "Add operator precedence comprehensive test fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #570 — Add operator precedence comprehensive test fixture

## Context
Python's operator precedence has 18+ levels. Incorrect precedence in the parser leads to silently wrong ASTs.

## Test cases (each should parse AND produce correct AST structure)
```python
# Ternary vs assignment
x = a if b else c

# Ternary vs lambda
f = lambda: a if b else c

# Boolean operators
a or b and c           # a or (b and c)
not a or b             # (not a) or b
not a and not b        # (not a) and (not b)

# Comparison chaining
a < b < c              # (a < b) and (b < c)
a < b > c == d         # chained
a is not b             # single operator
a not in b             # single operator

# Bitwise
a | b & c              # a | (b & c)
a ^ b | c & d          # a ^ (b | (c & d))... wait no
~a ** b                # (~a) ** b? or ~(a ** b)?

# Arithmetic
a + b * c              # a + (b * c)
-a ** b                # -(a ** b)  NOT (-a) ** b
a ** -b                # a ** (-b)
a ** b ** c            # a ** (b ** c) right-associative

# Await precedence
# await a ** b         # await (a ** b)

# Walrus
(x := a or b)          # x := (a or b)
(x := a if b else c)   # x := (a if b else c)

# Starred
*a, b = c              # (*a), b = c
a, *b = c              # a, (*b) = c

# Conditional expression nesting
a if b else c if d else e  # a if b else (c if d else e)

# Union type (PEP 604)
int | str | None        # (int | str) | None
```

## Task
Create `tests/fixtures/parse/edge_cases/operator_precedence.py` with `# RUN: parse`.
