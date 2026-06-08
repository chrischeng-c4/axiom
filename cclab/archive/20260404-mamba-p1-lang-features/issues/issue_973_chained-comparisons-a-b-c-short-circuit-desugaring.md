---
number: 973
title: "Chained comparisons — a < b < c short-circuit desugaring"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "parser-syntax"
---

# #973 — Chained comparisons — a < b < c short-circuit desugaring

Currently `a < b < c` compiles as `(a < b) < c` (wrong). Should desugar to `(a < b) and (b < c)` with short-circuit and single evaluation of `b`. Requires AST Compare node with multiple comparators.
