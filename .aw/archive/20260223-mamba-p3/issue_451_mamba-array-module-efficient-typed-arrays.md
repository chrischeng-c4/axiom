---
number: 451
title: "mamba: array module (efficient typed arrays)"
state: open
labels: [enhancement, crate:mamba, P3]
dependencies: [405]
---

# #451 — mamba: array module (efficient typed arrays)

## Description

Implement `array` module for memory-efficient typed arrays.

## Requirements

- R1: `array.array(typecode, initializer=())` — create typed array
- R2: Type codes: `'b'`, `'h'`, `'i'`, `'l'`, `'q'`, `'f'`, `'d'` (signed ints/floats)
- R3: Methods: `append`, `extend`, `insert`, `pop`, `remove`, `reverse`
- R4: `.tobytes()`, `.frombytes(data)` — binary conversion
- R5: `.tolist()` — convert to Python list
- R6: Buffer protocol support

## Dependencies

Depends on #405 (bytes type).

## Priority

P3 — used for binary data and numeric computation.
