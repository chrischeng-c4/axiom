---
number: 437
title: "mamba: weakref module"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #437 — mamba: weakref module

## Description

Implement `weakref` module for weak references — references that don't prevent garbage collection.

## Requirements

- R1: `weakref.ref(obj, callback=None)` — create weak reference
- R2: Calling weak ref returns object or None if collected
- R3: `weakref.WeakValueDictionary()` — dict with weak values
- R4: `weakref.WeakKeyDictionary()` — dict with weak keys
- R5: `weakref.WeakSet()` — set with weak references
- R6: `weakref.finalize(obj, func, *args)` — invoke callback when collected
- R7: `__weakref__` slot support on classes

## Priority

P2 — used internally by many frameworks for caching and observer patterns.
