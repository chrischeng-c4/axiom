---
number: 420
title: "mamba: missing builtins — map, filter, any, all, round, divmod, format"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #420 — mamba: missing builtins — map, filter, any, all, round, divmod, format

## Description

Several very commonly used Python builtins are not yet implemented.

## Requirements

### Iterating builtins
- R1: `map(fn, iterable)` — apply function to each element (lazy iterator)
- R2: `filter(fn, iterable)` — filter elements by predicate (lazy iterator)
- R3: `any(iterable)` — True if any element is truthy
- R4: `all(iterable)` — True if all elements are truthy

### Numeric builtins
- R5: `round(number, ndigits=None)` — round to n decimal places
- R6: `divmod(a, b)` — return (a // b, a % b) tuple

### Formatting
- R7: `format(value, format_spec="")` — format a value using __format__

### Iterator constructors
- R8: `iter(callable, sentinel)` — two-argument form
- R9: `next(iterator, default)` — with default value

## Priority

P1 — `any`, `all`, `map`, `filter` are among the most used Python builtins.
