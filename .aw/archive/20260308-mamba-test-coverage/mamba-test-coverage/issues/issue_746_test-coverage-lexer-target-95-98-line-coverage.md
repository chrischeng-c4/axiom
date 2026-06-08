---
number: 746
title: "Test coverage: Lexer — target 95–98% line coverage"
state: open
labels: [enhancement, P2, crate:mamba]
group: "parser-lexer-coverage"
---

# #746 — Test coverage: Lexer — target 95–98% line coverage

## Target
Line coverage: **95–98%**

## Scope
- `src/lexer/` — tokenization, string literals, indentation handling

## Approach
1. Measure per-file coverage
2. Cover all token types, escape sequences, string prefixes
3. Edge cases: mixed indentation, Unicode identifiers, nested strings
