---
number: 1129
title: "refactor(mamba): implement CPython 3.12 reference counting in JIT codegen"
state: open
labels: [priority:p0, crate:mamba, type:refactor]
group: "jit-refcount-audit"
---

# #1129 — refactor(mamba): implement CPython 3.12 reference counting in JIT codegen

## Problem

Mamba's JIT codegen does not emit `mb_retain`/`mb_release` calls. All `MbValue` are passed as raw `u64` without ownership semantics. This causes:

1. **Compile-time string constants leak** — `MbObject::new_str()` in `jit.rs` creates heap objects embedded as `iconst` immediates. These are never freed when the backend drops. (~4 call sites: str literals, bytes literals, getattr, setattr)

2. **Runtime container objects leak** — lists, dicts, tuples created during execution are tracked by the GC but never collected (GC disabled due to #1114)

3. **GC is disabled** — `gc.rs` `enabled: false` because JIT code doesn't register roots. Auto-collection caused heap-use-after-free (fixed in 9d66d5f9). Re-enabling requires either root scanning or proper refcounting.

4. **No object deallocation** — without retain/release, `ob_refcnt` stays at 1 forever. Objects are only freed by thread-local cleanup on thread exit.

## Progress (2026-03-27)

### Done (committed on `cclab-sdd-mamba-conformance`)

| Phase | Commit | Status |
|-------|--------|--------|
| **Immortal refcount** | `deedaf6e` | ✅ `IMMORTAL_REFCOUNT = u32::MAX`, `new_str_immortal()`, `new_bytes_immortal()` |
| **JIT wrappers** | `deedaf6e` | ✅ `mb_retain_value(u64)` / `mb_release_value(u64)` — NaN-box tag check, skip immortals |
| **Runtime symbols** | `deedaf6e` | ✅ Registered in `symbols.rs` |
| **Compile-time tracking** | `deedaf6e` | ✅ `compile_time_objects: Vec<*mut MbObject>` on backend, freed on Drop |
| **Container retain on store** | `3718d755` | ✅ `mb_list_append`, `mb_list_setitem`, `mb_dict_setitem`, `mb_set_add` retain stored values |
| **Cascading release on free** | `3718d755` | ✅ `release_contained_values()` in `mb_release` before `drop(Box::from_raw)` |
| **Cycle-safe sentinel** | `3718d755` | ✅ Set rc=IMMORTAL before releasing contained values to prevent re-entrant cycles |
| **GC sweep double-free fix** | `3718d755` | ✅ Check `tracked.remove()` result before freeing (cascading release may have already freed) |
| **Release-before-overwrite (ALL dest instructions)** | `16c1890f` | ✅ Emit `mb_release_value(old)` for every VReg-writing instruction (LoadConst, Call, BinOp, GetAttr, MakeList, etc.) |
| **Copy retain** | `deedaf6e` | ✅ `mb_retain_value(new)` after Copy (aliasing = both variables reference same object) |
| **Return cleanup** | `deedaf6e` | ✅ Release all I64 locals except return value at `Terminator::Return` |
| **`EMIT_REFCOUNT_CALLS` flag** | all above | ⚠️ Code written but **flag = false** — enabling causes heap-use-after-free |

### Blocked: Ownership Audit

Enabling `EMIT_REFCOUNT_CALLS = true` causes **heap-use-after-free** (confirmed by ASan). Root cause:

Runtime functions have **mixed return ownership semantics**:
- Some return **new references** (rc=1, caller owns): `mb_list_new()`, `mb_dict_new()`, `mb_str_concat()`
- Some return **borrowed references** (no incref, container still owns): `mb_list_getitem()`, `mb_dict_getitem()`, `mb_getattr()`

When the JIT releases ALL local variables at function return, borrowed references get released → refcount drops to 0 → **freed while container still holds the pointer** → use-after-free on next access.

**Fix needed:** Audit every `mb_*` runtime function and classify as:
1. **New reference** (caller must release) — no change needed
2. **Borrowed reference** (caller must NOT release) — must add `mb_retain` before returning so the caller gets an owned reference

This is the same distinction as CPython's `PyList_GetItem` (borrowed) vs `PyObject_GetAttr` (new reference).

### Not Started

| Phase | Status |
|-------|--------|
| **Runtime ownership audit** | ❌ Classify all `mb_*` return values as new/borrowed, add retain for borrowed |
| **Enable `EMIT_REFCOUNT_CALLS = true`** | ❌ Blocked by audit |
| **Re-enable GC** | ❌ Blocked by flag — `gc.rs` `enabled: false` |

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

## Key Files

| File | Changes |
|------|---------|
| `runtime/rc.rs` | IMMORTAL_REFCOUNT, immortal constructors, JIT wrappers, cascading release, retain_if_ptr/release_if_ptr |
| `runtime/symbols.rs` | mb_retain_value / mb_release_value registered |
| `codegen/cranelift/jit.rs` | Compile-time tracking + Drop, release-before-overwrite, Copy retain, return cleanup |
| `codegen/cranelift/mod.rs` | EMIT_REFCOUNT_CALLS flag, VarAlloc type tracking, emit_terminator release |
| `runtime/list_ops.rs` | mb_list_append/setitem retain |
| `runtime/dict_ops.rs` | mb_dict_setitem retain + release old |
| `runtime/set_ops.rs` | mb_set_add retain |
| `runtime/gc.rs` | Sweep double-free prevention |

## Related
- #1114 — SIGBUS crash (fixed by disabling GC, root cause: GC freed live objects)
- #653 — `gc` stdlib module (unblocked once GC re-enabled)
- #1009 — concurrency semantics (atomic refcount already in place)
