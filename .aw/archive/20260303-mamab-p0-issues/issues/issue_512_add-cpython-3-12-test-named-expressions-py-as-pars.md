---
number: 512
title: "Add CPython 3.12 test_named_expressions.py as parse-only fixture (walrus operator)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #512 — Add CPython 3.12 test_named_expressions.py as parse-only fixture (walrus operator)

## Context
Tests the walrus operator (`:=`, PEP 572). This is a relatively new syntax feature that needs thorough parser coverage.

## Task
1. Download `test_named_expressions.py` from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/test_named_expressions.py`

## Acceptance
- Fixture parses without errors
- `cargo test --test fixture_tests` passes
