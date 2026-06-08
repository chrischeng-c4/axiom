---
number: 515
title: "Add CPython 3.12 type system fixtures (test_type_aliases, test_type_annotations, test_type_comments, test_type_params, test_pep646_syntax)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #515 — Add CPython 3.12 type system fixtures (test_type_aliases, test_type_annotations, test_type_comments, test_type_params, test_pep646_syntax)

## Context
Python's type system syntax has grown significantly (PEP 484, 526, 604, 612, 646, 695). These files test:
- `test_type_aliases.py` — PEP 695 `type X = ...` syntax
- `test_type_annotations.py` — variable/function annotations
- `test_type_comments.py` — `# type:` comments
- `test_type_params.py` — PEP 695 `def f[T]()` syntax
- `test_pep646_syntax.py` — PEP 646 `*Ts` variadic generics

## Task
For each file:
1. Download from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax, strip unittest runtime logic
3. Add `# RUN: parse` directive
4. Place in `tests/fixtures/parse/cpython/stdlib/`

## Acceptance
- All 5 fixtures parse without errors
- `cargo test --test fixture_tests` passes
