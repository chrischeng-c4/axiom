---
number: 438
title: "mamba: inspect module (runtime introspection)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #438 — mamba: inspect module (runtime introspection)

## Description

Implement `inspect` module for runtime object introspection.

## Requirements

- R1: `inspect.isfunction(obj)`, `inspect.ismethod(obj)`, `inspect.isclass(obj)`
- R2: `inspect.getmembers(obj, predicate=None)` — list all members
- R3: `inspect.signature(callable)` — get function signature
- R4: `inspect.getsource(obj)` — get source code (requires source mapping)
- R5: `inspect.getfile(obj)` — get source file
- R6: `inspect.stack()` — current call stack
- R7: `inspect.currentframe()` — current frame

## Priority

P2 — used by debugging tools, documentation generators, and many libraries.
