---
number: 453
title: "mamba: complex number full operations"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #453 — mamba: complex number full operations

## Description

The `complex` type is parsed (ComplexLit in AST) but operations are limited. Implement full complex number support.

## Requirements

- R1: `complex(real, imag)` constructor
- R2: Arithmetic: +, -, *, /, ** with complex operands
- R3: `.real`, `.imag` properties
- R4: `abs(z)` — magnitude
- R5: `z.conjugate()` method
- R6: Mixed arithmetic with int/float
- R7: `cmath` module: `cmath.sqrt`, `cmath.exp`, `cmath.log`, `cmath.polar`, `cmath.rect`, `cmath.phase`

## Priority

P3 — scientific computing, rarely needed in general programs.
