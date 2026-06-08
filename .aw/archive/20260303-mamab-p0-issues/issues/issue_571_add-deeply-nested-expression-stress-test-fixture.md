---
number: 571
title: "Add deeply nested expression stress test fixture"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #571 — Add deeply nested expression stress test fixture

## Context
Parsers often have stack overflow or performance issues with deeply nested expressions. Need stress tests.

## Test cases
- Deeply nested parentheses: `((((((((((x))))))))))`
- Deeply nested function calls: `f(f(f(f(f(x)))))`
- Deeply nested list literals: `[[[[[x]]]]]`
- Deeply nested dict literals
- Deeply nested comprehensions
- Deeply nested ternary: `a if b else c if d else e if f else g`
- Deeply nested boolean: `a and b and c and d and ...` (100+ levels)
- Deeply nested attribute access: `a.b.c.d.e.f.g.h`
- Deeply nested subscript: `a[b[c[d[e]]]]`
- Long argument lists: `f(a1, a2, ..., a200)`
- Long unpacking: `a1, a2, ..., a100 = iterable`
- Large match statement with 50+ case clauses
- Very long f-string with many expressions
- Deeply nested class definitions
- 100+ chained method calls

## Task
Create `tests/fixtures/parse/stress/` directory with fixtures targeting specific depth/size limits.

## Acceptance
- All fixtures parse without stack overflow
- Parse time remains reasonable (< 1 second per file)
