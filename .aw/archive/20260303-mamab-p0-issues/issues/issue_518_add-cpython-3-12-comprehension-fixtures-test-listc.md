---
number: 518
title: "Add CPython 3.12 comprehension fixtures (test_listcomps, test_dictcomps, test_setcomps)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #518 — Add CPython 3.12 comprehension fixtures (test_listcomps, test_dictcomps, test_setcomps)

## Context
Comprehensions are syntactically complex (nested loops, conditions, walrus operator inside, async comprehensions). Current mamba coverage: only `comprehensions.py` (5 lines).

## Files
- `test_listcomps.py` — list comprehensions
- `test_dictcomps.py` — dict comprehensions
- `test_setcomps.py` — set comprehensions

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 3 fixtures parse without errors
- `cargo test --test fixture_tests` passes
