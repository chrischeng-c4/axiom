---
change: mamba-refcount-jit
group: refcount-jit
date: 2026-03-27
---

# Requirements

Implement CPython 3.12 reference counting in JIT codegen. (1) Register mb_retain/mb_release as JIT symbols in symbols.rs. (2) Emit mb_release before variable reassignment and mb_retain for new heap pointer values in jit.rs emit_inst. (3) Emit mb_release for all live local variables at function return in emit_terminator. (4) Add IMMORTAL_REFCOUNT to rc.rs — compile-time string/bytes constants use immortal refcount so they are never freed. (5) Guard mb_retain/mb_release to skip immortal objects. (6) Re-enable GC auto-collection in gc.rs once refcounting is correct. NaN-boxing tag check: only retain/release TAG_PTR (tag=0) values.
