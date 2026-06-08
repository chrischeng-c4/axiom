---
number: 754
title: "Py3.12 conformance: Object model — class, MRO, descriptors, metaclass"
state: open
labels: [enhancement, P0, crate:mamba]
group: "mamba-conformance-xfail"
---

# #754 — Py3.12 conformance: Object model — class, MRO, descriptors, metaclass

## Parent

Part of #750

## Goal

Verify mamba object model matches CPython 3.12 semantics.

## Scope

- [ ] Class creation: `class Foo(Bar):` with single/multiple inheritance
- [ ] MRO: C3 linearization matches `Foo.__mro__`
- [ ] Descriptors: `__get__`, `__set__`, `__delete__` protocol
- [ ] Properties: `@property`, `@x.setter`, `@x.deleter`
- [ ] Metaclass: `class Meta(type):`, `__init_subclass__`
- [ ] `__slots__` behavior
- [ ] `super()` with zero-arg and explicit forms
- [ ] `__new__` vs `__init__` ordering
- [ ] Attribute lookup order: instance → class → bases → `__getattr__`

## Current State

Basic class/instance works. MRO, descriptors, metaclass need conformance verification.
