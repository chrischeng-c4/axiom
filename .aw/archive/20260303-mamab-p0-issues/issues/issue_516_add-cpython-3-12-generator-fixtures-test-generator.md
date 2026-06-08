---
number: 516
title: "Add CPython 3.12 generator fixtures (test_generators, test_genexps, test_generator_stop, test_yield_from)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #516 — Add CPython 3.12 generator fixtures (test_generators, test_genexps, test_generator_stop, test_yield_from)

## Context
Generators are central to Python and involve complex syntax: `yield`, `yield from`, generator expressions, `StopIteration` handling. These are essential for parser coverage.

## Files
- `test_generators.py` — generator functions, yield expressions
- `test_genexps.py` — generator expressions `(x for x in ...)`
- `test_generator_stop.py` — StopIteration edge cases
- `test_yield_from.py` — `yield from` delegation

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 4 fixtures parse without errors
- `cargo test --test fixture_tests` passes
