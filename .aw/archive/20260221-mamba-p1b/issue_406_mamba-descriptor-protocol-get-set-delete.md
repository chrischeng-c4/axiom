---
number: 406
title: "mamba: descriptor protocol (__get__/__set__/__delete__)"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #406 — mamba: descriptor protocol (__get__/__set__/__delete__)

## Description

Implement the descriptor protocol, which is the foundation for `@property`, `@classmethod`, `@staticmethod`, and many advanced Python patterns.

## Requirements

- R1: Data descriptors — objects with `__get__` and `__set__` (or `__delete__`)
- R2: Non-data descriptors — objects with only `__get__`
- R3: Descriptor invocation during attribute access (`__getattribute__` protocol)
- R4: Built-in descriptor types:
  - `property(fget, fset, fdel, doc)` — wraps getter/setter/deleter
  - `classmethod` — binds method to class instead of instance
  - `staticmethod` — removes implicit first argument
- R5: `__set_name__` hook called at class creation time

## Dependencies

Blocks #384 (property/classmethod/staticmethod decorators).

## Priority

P1 — required for idiomatic Python OOP patterns.
