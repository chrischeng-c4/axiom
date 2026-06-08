---
change: mamba-refcount-jit
group: refcount-jit
date: 2026-03-27
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
JIT codegen never emits mb_retain/mb_release — all heap objects leak. GC disabled because no root registration. #1129

### Success Criteria
(not provided)

### Boundary
(not provided)

