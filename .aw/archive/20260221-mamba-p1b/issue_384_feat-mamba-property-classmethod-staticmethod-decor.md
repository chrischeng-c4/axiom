---
number: 384
title: "feat(mamba): property, classmethod, staticmethod decorators"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #384 — feat(mamba): property, classmethod, staticmethod decorators

## Summary
Implement the three core descriptor decorators for Python classes.

## Required
- `@property` — getter, setter, deleter
- `@classmethod` — receives `cls` as first arg instead of `self`
- `@staticmethod` — no implicit first arg
- Descriptor protocol: `__get__`, `__set__`, `__delete__`

## Implementation Notes
- Decorators are already parsed; need runtime support in `class.rs`
- Property needs descriptor wrapper objects
- classmethod/staticmethod need special callable wrappers that modify how `self`/`cls` is passed
