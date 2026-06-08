---
number: 745
title: "Test coverage: Parser — target 95–98% line coverage"
state: open
labels: [enhancement, P2, crate:mamba]
group: "parser-lexer-coverage"
---

# #745 — Test coverage: Parser — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/parser/` — tokenization, AST construction, error recovery

## Current
- 85.4 T/KLoC (already relatively high)
- Needs line coverage measurement to identify gaps

## Approach
1. Identify uncovered parser rules via tarpaulin
2. Add tests for error recovery paths, edge case syntax
3. Negative test fixtures for all parse error types
