---
change: mamba-p1-lang-features
group: parser-syntax
date: 2026-04-04
---

# Requirements

Parser-level syntax additions: (1) Dict literal unpacking {**d1, **d2, key: val} — new AST node DictUnpack, parser changes in expr_compound.rs. (2) Chained comparisons a < b < c — desugar to (a < b) and (b < c) with short-circuit, parser changes in expressions.rs. Both are parser→AST→lowering changes.
