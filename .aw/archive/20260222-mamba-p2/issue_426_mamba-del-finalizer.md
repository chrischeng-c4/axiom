---
number: 426
title: "mamba: __del__ finalizer"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #426 — mamba: __del__ finalizer

## Description

Implement `__del__` finalizer support — called when object is garbage collected.

## Requirements

- R1: `__del__(self)` called when refcount drops to zero
- R2: Handle resurrection (object re-referenced during __del__)
- R3: Suppress exceptions raised in __del__ (print warning to stderr)
- R4: Integration with GC cycle collector

## Priority

P2 — used for resource cleanup (file handles, connections).
