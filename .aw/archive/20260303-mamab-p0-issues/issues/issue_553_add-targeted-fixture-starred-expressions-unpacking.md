---
number: 553
title: "Add targeted fixture: starred expressions & unpacking in all contexts"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #553 — Add targeted fixture: starred expressions & unpacking in all contexts

## Context
Starred expressions appear in many contexts beyond simple unpacking.

## Test cases to cover
- Assignment: `a, *b, c = [1, 2, 3, 4]`
- For loop: `for a, *b in pairs:`
- Function args: `def f(*args, **kwargs):`
- Call: `f(*a, **b)`, `f(*a, *b)` (PEP 448)
- List/tuple/set literal: `[*a, *b]`, `(*a, *b)`, `{*a, *b}`
- Dict literal: `{**a, **b}`
- Return: `return *a,`
- Yield: `yield *a`
- Nested unpacking: `(a, (b, *c)) = ...`

## Task
Create `tests/fixtures/parse/edge_cases/starred_expressions.py` with `# RUN: parse`.
