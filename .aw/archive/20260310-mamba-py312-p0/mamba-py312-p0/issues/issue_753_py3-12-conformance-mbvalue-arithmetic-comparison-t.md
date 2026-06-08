---
number: 753
title: "Py3.12 conformance: MbValue arithmetic, comparison & truthiness"
state: open
labels: [enhancement, P0, crate:mamba]
group: "py312-conformance"
---

# #753 — Py3.12 conformance: MbValue arithmetic, comparison & truthiness

## Parent

Part of #750

## Goal

Verify NaN-boxed MbValue arithmetic, comparison, and truthiness match CPython 3.12 exactly.

## Scope

- [ ] int arithmetic: +, -, *, //, /, %, **, unary -
- [ ] float arithmetic: same ops, IEEE 754 edge cases (inf, nan, -0.0)
- [ ] complex arithmetic: +, -, *, /, abs(), conjugate()
- [ ] Mixed-type promotion: int+float, int+complex, float+complex
- [ ] Comparison: ==, !=, <, >, <=, >= across types
- [ ] Truthiness: bool(0), bool(""), bool([]), bool(None), bool(1), etc.
- [ ] round(), abs(), pow(), divmod() edge cases

## Current State

Basic arithmetic works. No systematic comparison against CPython 3.12 behavior for edge cases.
