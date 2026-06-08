---
change: mamba-jit-memory
group: jit-memory
date: 2026-03-26
status: skipped
---

# Post-Clarifications

## Scope Summary

### Problem
cranelift-jit uses alloc::alloc + region::protect(mprotect) for JIT pages. On macOS aarch64, this causes EXC_BAD_ACCESS code=257 after ~100 cross-thread JIT executions. CPython 3.14, V8, JavaScriptCore all use MAP_JIT + pthread_jit_write_protect_np on Apple Silicon.

### Success Criteria
(not provided)

### Boundary
(not provided)

