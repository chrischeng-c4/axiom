---
number: 409
title: "mamba: starred unpacking codegen (a, *b, c = iterable)"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #409 — mamba: starred unpacking codegen (a, *b, c = iterable)

## Description

Starred unpacking is parsed but not fully wired through codegen. This is a very common Python pattern.

## Requirements

- R1: `a, *b, c = [1, 2, 3, 4, 5]` → a=1, b=[2,3,4], c=5
- R2: `first, *rest = iterable` — common "head/tail" pattern
- R3: `*init, last = iterable` — common "init/last" pattern
- R4: Nested unpacking: `(a, b), c = [1, 2], 3`
- R5: Unpacking in for loops: `for a, *b in pairs:`
- R6: `*args` unpacking in function calls: `f(*args, **kwargs)`
- R7: `**kwargs` unpacking in function calls

## Current State

- Parser handles starred expressions correctly
- HIR/MIR lowering and codegen not fully implemented for these patterns

## Priority

P1 — extremely common Python pattern.
