---
number: 552
title: "Add targeted fixture: match statement advanced patterns"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #552 — Add targeted fixture: match statement advanced patterns

## Context
PEP 634 match/case has many pattern types. Current `match_stmt.py` is only 7 lines.

## Test cases to cover
- Literal patterns (int, float, str, bool, None, complex)
- Capture patterns
- Wildcard `_`
- Value patterns: `Color.RED`
- Sequence patterns: `[a, *rest, b]`
- Mapping patterns: `{"key": value, **rest}`
- Class patterns: `Point(x=0, y=0)`
- OR patterns: `1 | 2 | 3`
- AS patterns: `pattern as name`
- Guard clauses: `case x if x > 0:`
- Nested/combined patterns
- Pattern matching with walrus
- Complex subject expressions

## Task
Create `tests/fixtures/parse/edge_cases/match_advanced.py` with `# RUN: parse`.
