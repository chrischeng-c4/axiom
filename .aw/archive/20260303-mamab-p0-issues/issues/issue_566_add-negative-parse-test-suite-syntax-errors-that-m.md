---
number: 566
title: "Add negative parse test suite: syntax errors that MUST be rejected"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #566 — Add negative parse test suite: syntax errors that MUST be rejected

## Context
A parser that accepts everything is useless. We need to verify that mamba **rejects** invalid syntax with clear errors. CPython's `test_syntax.py` covers some cases, but we need comprehensive negative testing.

## Test cases to cover (all should use `# RUN: parse` + `# EXPECT-ERROR:`)
- Missing colons: `if True print()`, `def f() pass`
- Invalid assignment targets: `1 = x`, `f() = x`, `x + y = z`
- Invalid augmented assignment: `x + y += 1`
- Invalid del targets: `del 1`, `del f()`
- Invalid starred: `*x = 1` (outside unpacking), `**x` in assignment
- Break/continue outside loop
- Return outside function
- Yield outside function
- Nonlocal/global in module scope edge cases
- Duplicate arguments: `def f(a, a):`
- Positional after keyword: `f(a=1, b)`
- Multiple `*args`: `def f(*a, *b):`
- Multiple `**kwargs`: `def f(**a, **b):`
- Assignment expression in wrong context
- `except*` mixed with `except` in same `try`
- Invalid decorator targets
- Empty match/case body
- Mismatched brackets
- Unterminated strings/f-strings

## Task
Create `tests/fixtures/parse/negative/` directory with multiple fixture files, each using `# EXPECT-ERROR:` directive.

## Note
This requires the fixture harness to support error expectation matching properly.
