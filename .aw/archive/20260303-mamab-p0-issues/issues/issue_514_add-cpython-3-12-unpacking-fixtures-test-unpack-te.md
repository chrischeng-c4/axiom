---
number: 514
title: "Add CPython 3.12 unpacking fixtures (test_unpack, test_unpack_ex)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #514 — Add CPython 3.12 unpacking fixtures (test_unpack, test_unpack_ex)

## Context
Tests tuple unpacking and extended unpacking (PEP 3132: `a, *b, c = iterable`). Starred expressions in various contexts are a common source of parser bugs.

## Files
- `test_unpack.py`
- `test_unpack_ex.py`

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- Both fixtures parse without errors
- `cargo test --test fixture_tests` passes
