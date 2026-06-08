---
number: 550
title: "Add targeted fixture: walrus operator (:=) edge cases"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #550 — Add targeted fixture: walrus operator (:=) edge cases

## Context
The walrus operator (PEP 572) has many syntactic edge cases that need thorough testing beyond what `test_named_expressions.py` covers.

## Test cases to cover
- `:=` in `if`, `while`, `for`, comprehensions
- Nested walrus: `(a := (b := 1))`
- Walrus in f-strings: `f"{(x := 10)}"`
- Walrus in lambda: `lambda: (x := 1)`
- Walrus in assert: `assert (x := f())`
- Walrus with complex targets
- Invalid walrus positions (EXPECT-ERROR cases)

## Task
Create `tests/fixtures/parse/edge_cases/walrus_operator.py` with `# RUN: parse` directive, covering all edge cases.
