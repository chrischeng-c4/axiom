---
number: 414
title: "mamba: copy module (copy/deepcopy)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #414 — mamba: copy module (copy/deepcopy)

## Description

Implement `copy` module for shallow and deep copying of objects.

## Requirements

- R1: `copy.copy(obj)` — shallow copy
- R2: `copy.deepcopy(obj)` — deep recursive copy
- R3: `__copy__` and `__deepcopy__` dunder method hooks
- R4: Handle circular references in deepcopy

## Priority

P2 — commonly needed for data manipulation.
