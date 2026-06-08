---
number: 1129
title: "refactor(mamba): implement CPython 3.12 reference counting in JIT codegen"
state: open
labels: [priority:p0, crate:mamba, type:refactor]
group: "refcount-jit"
---

# #1129 — refactor(mamba): implement CPython 3.12 reference counting in JIT codegen

## Problem

Mamba's JIT codegen does not emit `mb_retain`/`mb_release` calls. All `MbValue` are passed as raw `u64` without ownership semantics. This causes:

1. **Compile-time string constants leak** — `MbObject::new_str()` in `jit.rs` creates heap objects embedded as `iconst` immediates. These are never freed when the backend drops. (~4 call sites: str literals, bytes literals, getattr, setattr)

2. **Runtime container objects leak** — lists, dicts, tuples created during execution are tracked by the GC but never collected (GC disabled due to #1114)

3. **GC is disabled** — `gc.rs` `enabled: false` because JIT code doesn't register roots. Auto-collection caused heap-use-after-free (fixed in 9d66d5f9). Re-enabling requires either root scanning or proper refcounting.

4. **No object deallocation** — without retain/release, `ob_refcnt` stays at 1 forever. Objects are only freed by thread-local cleanup on thread exit.

## CPython 3.12 Reference Model

```
Py_INCREF(obj)  — on assignment, function arg pass, container insert
Py_DECREF(obj)  — on reassignment, scope exit, container remove
    → if refcnt == 0: tp_dealloc(obj) — immediate free
    → container objects: gc_track / gc_untrack

Immortal objects (PEP 683): _Py_IMMORTAL_REFCNT
    — None, True, False, small ints, interned strings
    — never incremented/decremented, never freed
```

## Proposed Implementation

### Phase 1: JIT emit retain/release (P0)
- Emit `mb_retain(val)` when storing a heap pointer to a variable
- Emit `mb_release(val)` when variable goes out of scope or is reassigned
- Use NaN-boxing tag bits to skip retain/release for non-pointer values (int, float, bool, None)
- Compile-time constants use immortal refcount (never release)

### Phase 2: Track compile-time allocations (P0)
- `CraneliftJitBackend` tracks all `MbObject::new_str()` / `new_bytes()` pointers
- On `Drop`, release all tracked compile-time objects (after `free_memory()`)

### Phase 3: Re-enable GC for cycle detection (P1)
- With proper refcounting, most objects are freed by Py_DECREF
- GC only needs to handle cycles (list containing itself, etc.)
- Conservative stack scanning as root source (no stack maps needed initially)
- Re-enable `gc.enabled = true` with proper roots

## Impact

This is foundational — the longer it's deferred, the more code builds on the assumption that objects are never freed. Every new runtime function, every new container type, every new builtin inherits the leak. Early fix means:
- Correct memory behavior for REPL sessions (currently leaks indefinitely)
- Prerequisite for long-running programs
- Matches CPython 3.12 `__del__` / weak reference semantics
- Unblocks `gc` stdlib module (#653)

## Related
- #1114 — SIGBUS crash (fixed by disabling GC, root cause: no JIT roots)
- #653 — `gc` stdlib module
- #1009 — concurrency semantics (atomic refcount already in place)
