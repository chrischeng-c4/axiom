---
number: 551
title: "Add targeted fixture: complex f-string nesting edge cases"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #551 — Add targeted fixture: complex f-string nesting edge cases

## Context
F-strings (PEP 498, enhanced in 3.12 PEP 701) have complex nesting rules that stress parsers.

## Test cases to cover
- Nested f-strings: `f"{f'{x}'}"`
- F-string with format spec: `f"{x:.{precision}f}"`
- F-string with conversions: `f"{x!r:.10}"`
- F-string with dict access: `f"{d['key']}"`
- F-string with walrus: `f"{(x := 10)}"`
- Multi-line f-strings
- F-string with backslash (3.12 allows `f"{'\n'}"`)
- Triple-quoted f-strings
- Raw f-strings: `rf"{x}"`
- Deeply nested: `f"{f'{f"{x}"}'}"` (3.12 allows arbitrary nesting)

## Task
Create `tests/fixtures/parse/edge_cases/fstring_nesting.py` with `# RUN: parse`.
