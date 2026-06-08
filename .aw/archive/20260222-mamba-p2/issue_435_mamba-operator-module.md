---
number: 435
title: "mamba: operator module"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #435 — mamba: operator module

## Description

Implement `operator` module — function equivalents of operators. Used with `map`, `sorted`, `functools.reduce`.

## Requirements

- R1: Arithmetic: `operator.add`, `sub`, `mul`, `truediv`, `floordiv`, `mod`, `pow`, `neg`, `pos`, `abs`
- R2: Comparison: `operator.eq`, `ne`, `lt`, `le`, `gt`, `ge`
- R3: Logic: `operator.and_`, `or_`, `not_`, `xor`
- R4: Item access: `operator.getitem`, `setitem`, `delitem`
- R5: Attribute access: `operator.attrgetter(attr)`, `itemgetter(item)`, `methodcaller(name)`
- R6: `operator.contains`, `countOf`, `indexOf`

## Priority

P2 — commonly used as key functions with sorted/map/reduce.
