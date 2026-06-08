---
number: 513
title: "Add CPython 3.12 test_patma.py as parse-only fixture (pattern matching)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #513 — Add CPython 3.12 test_patma.py as parse-only fixture (pattern matching)

## Context
`test_patma.py` is CPython's comprehensive pattern matching test (PEP 634). It covers all match/case patterns: literal, capture, wildcard, value, sequence, mapping, class, OR, AS, guard.

This is critical — mamba already has `match_stmt.py` (7 lines) which is far too thin.

## Task
1. Download `test_patma.py` from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/test_patma.py`

## Acceptance
- Fixture parses without errors
- `cargo test --test fixture_tests` passes
