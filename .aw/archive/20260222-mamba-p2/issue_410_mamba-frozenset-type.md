---
number: 410
title: "mamba: frozenset type"
state: open
labels: [enhancement, P2, crate:mamba]
dependencies: [386]
---

# #410 — mamba: frozenset type

## Description

Implement `frozenset` — immutable set type. Required for using sets as dict keys or elements of other sets.

## Requirements

- R1: `frozenset()` and `frozenset(iterable)` constructors
- R2: Set operations: `|` (union), `&` (intersection), `-` (difference), `^` (symmetric difference)
- R3: Methods: `union`, `intersection`, `difference`, `symmetric_difference`, `issubset`, `issuperset`, `isdisjoint`, `copy`
- R4: `__hash__` — hashable (unlike set)
- R5: `in` membership test, `len()`, iteration

## Dependencies

Depends on #386 (set type) — share implementation, frozenset adds immutability + hashability.

## Priority

P2 — needed for advanced data structures.
