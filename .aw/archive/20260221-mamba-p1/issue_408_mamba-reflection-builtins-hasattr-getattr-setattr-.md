---
number: 408
title: "mamba: reflection builtins (hasattr/getattr/setattr/delattr/callable)"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #408 — mamba: reflection builtins (hasattr/getattr/setattr/delattr/callable)

## Description

Implement missing reflection/introspection builtins that many Python programs depend on.

## Requirements

- R1: `hasattr(obj, name)` — return True if attribute exists (catches AttributeError)
- R2: `getattr(obj, name)` and `getattr(obj, name, default)` — get attribute with optional default
- R3: `setattr(obj, name, value)` — set attribute on object
- R4: `delattr(obj, name)` — delete attribute from object
- R5: `callable(obj)` — return True if obj has `__call__`
- R6: `vars(obj)` — return `__dict__` of object (optional, lower priority)
- R7: `dir(obj)` — list attributes (optional, lower priority)

## Priority

P1 — very commonly used in real Python programs and by many libraries.
