---
number: 511
title: "Add CPython 3.12 operator & call expression fixtures (test_augassign, test_binop, test_call, test_extcall)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #511 — Add CPython 3.12 operator & call expression fixtures (test_augassign, test_binop, test_call, test_extcall)

## Context
These files test core expression parsing: augmented assignment (`+=`, `-=`, etc.), binary operators, function call syntax, and extended call syntax (PEP 448 `f(*a, **kw)`).

## Files
- `test_augassign.py`
- `test_binop.py`
- `test_call.py`
- `test_extcall.py`

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 4 fixtures parse without errors
- `cargo test --test fixture_tests` passes
