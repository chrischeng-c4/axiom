---
number: 510
title: "Add CPython 3.12 test_grammar.py as parse-only fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #510 — Add CPython 3.12 test_grammar.py as parse-only fixture

## Context
`test_grammar.py` is CPython's comprehensive grammar test — it exercises nearly every Python grammar production rule. This is the single most valuable file for parser conformance testing.

## Task
1. Download `test_grammar.py` from CPython 3.12 `Lib/test/`
2. Extract parse-able syntax (strip `unittest` runtime logic that requires stdlib imports beyond what mamba supports)
3. Add `# RUN: parse` directive at top
4. Place in `tests/fixtures/parse/cpython/stdlib/test_grammar.py`

## Acceptance
- `cargo test --test fixture_tests` passes with the new fixture
- File parses without errors

Part of the CPython 3.12 parse conformance initiative.
