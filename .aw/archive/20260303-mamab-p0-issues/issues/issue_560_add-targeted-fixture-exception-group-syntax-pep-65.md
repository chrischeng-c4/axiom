---
number: 560
title: "Add targeted fixture: exception group syntax (PEP 654)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #560 — Add targeted fixture: exception group syntax (PEP 654)

## Context
PEP 654 (Python 3.11) introduced `except*` and `ExceptionGroup`.

## Test cases to cover
- Basic `except*`: `try: ... except* ValueError as eg: ...`
- Multiple `except*` clauses
- Nested exception groups
- `except*` with tuple: `except* (TypeError, ValueError):`
- `except*` combined with regular `except`
- Raise ExceptionGroup
- ExceptionGroup nesting
- `except*` with walrus (if applicable)

## Task
Create `tests/fixtures/parse/edge_cases/exception_groups.py` with `# RUN: parse`.
