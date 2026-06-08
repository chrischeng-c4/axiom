---
number: 411
title: "mamba: __slots__ support"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #411 — mamba: __slots__ support

## Description

Implement `__slots__` for memory-optimized classes. Prevents `__dict__` creation and restricts attribute names.

## Requirements

- R1: `__slots__ = ('x', 'y')` class variable declaration
- R2: Slot-based attribute storage (no `__dict__` on instances)
- R3: AttributeError for undeclared attributes
- R4: Inheritance with slots (slot merging from bases)
- R5: `__slots__ = ()` for mixin classes

## Priority

P2 — performance optimization, used by many libraries.
