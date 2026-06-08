---
number: 425
title: "mamba: __format__ protocol and f-string debug syntax (f\"{x=}\")"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #425 — mamba: __format__ protocol and f-string debug syntax (f"{x=}")

## Description

Implement the `__format__` protocol and `f"{x=}"` debug syntax.

## Requirements

### __format__ protocol
- R1: `__format__(self, format_spec)` dunder method on all types
- R2: Default `__format__` delegates to `__str__` when format_spec is empty
- R3: Int format specs: `d`, `b`, `o`, `x`, `X`, `n`, `,`, `_`
- R4: Float format specs: `f`, `e`, `E`, `g`, `G`, `.Nf`, `%`
- R5: Alignment: `<`, `>`, `^`, fill character
- R6: Width, sign (`+`, `-`, ` `), `#` alternate form, `0` padding

### f-string debug syntax (PEP 572)
- R7: `f"{expr=}"` → prints `"expr=<value>"` — self-documenting expressions
- R8: `f"{expr=!r}"` — with repr conversion
- R9: `f"{expr=:.2f}"` — with format spec

## Dependencies

Blocks #388 (f-string format specifiers) partially.

## Priority

P2 — commonly used for formatted output.
