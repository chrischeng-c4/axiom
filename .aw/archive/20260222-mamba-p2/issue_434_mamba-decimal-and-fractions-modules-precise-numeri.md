---
number: 434
title: "mamba: decimal and fractions modules (precise numerics)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #434 — mamba: decimal and fractions modules (precise numerics)

## Description

Implement `decimal` and `fractions` modules for precise numeric computation.

## Requirements

### decimal
- R1: `Decimal(value)` constructor from string/int/float
- R2: Arithmetic: +, -, *, /, //, %, **
- R3: Comparison operators
- R4: `decimal.getcontext()` — precision, rounding mode
- R5: `Decimal.quantize(exp)` — round to specific decimal places

### fractions
- R6: `Fraction(numerator, denominator)` constructor
- R7: `Fraction(string)` — parse "3/4" or "0.75"
- R8: Arithmetic with automatic simplification
- R9: `.numerator`, `.denominator` properties
- R10: `Fraction.from_float(f)`, `Fraction.from_decimal(d)`

## Priority

P2 — needed for financial and scientific computing.
