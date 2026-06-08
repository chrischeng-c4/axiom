---
number: 412
title: "mamba: enum module runtime support"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #412 — mamba: enum module runtime support

## Description

The parser supports enum definitions, but runtime `enum` module support is missing. Python's `enum` module is widely used.

## Requirements

- R1: `class Color(Enum): RED = 1; GREEN = 2; BLUE = 3`
- R2: `Color.RED`, `Color['RED']`, `Color(1)` access patterns
- R3: `.name` and `.value` properties
- R4: Iteration: `for c in Color:`
- R5: `IntEnum`, `StrEnum` variants
- R6: `@unique` decorator
- R7: `auto()` for automatic values
- R8: Membership: `Color.RED in Color`

## Priority

P2 — very commonly used in modern Python.
