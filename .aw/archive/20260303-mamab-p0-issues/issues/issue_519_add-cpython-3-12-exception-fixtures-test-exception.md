---
number: 519
title: "Add CPython 3.12 exception fixtures (test_exceptions, test_baseexception, test_exception_group, test_exception_hierarchy, test_exception_variations)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #519 — Add CPython 3.12 exception fixtures (test_exceptions, test_baseexception, test_exception_group, test_exception_hierarchy, test_exception_variations)

## Context
Exception handling syntax includes `try/except/else/finally`, `raise from`, exception groups (PEP 654 `except*`), and chained exceptions. These are critical for parser correctness.

## Files
- `test_exceptions.py` — comprehensive exception tests
- `test_baseexception.py` — BaseException hierarchy
- `test_exception_group.py` — PEP 654 ExceptionGroup
- `test_exception_hierarchy.py` — exception class hierarchy
- `test_exception_variations.py` — edge cases

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 5 fixtures parse without errors
- `cargo test --test fixture_tests` passes
