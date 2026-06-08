---
number: 1084
title: "fix(mamba): @decorator return value is None — calling convention mismatch"
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "mamba-runtime-bugs"
---

# #1084 — fix(mamba): @decorator return value is None — calling convention mismatch

## Problem

Calling a decorated function returns `None` instead of the actual return value.

```python
def identity(f):
    return f

@identity
def foo():
    return 99

print(foo())  # Expected: 99, Got: None
```

## Root Cause

Calling convention mismatch between JIT-compiled functions and `mb_call0` dynamic dispatch:

- JIT functions return raw `i64` (NaN-boxed value)
- `mb_call0` in `runtime/class.rs:1956` transmutes the function pointer to `fn() -> MbValue`
- The MIR lowering at `hir_to_mir.rs:3479` then `box_operand` on the result, potentially double-boxing

The decorator application itself works correctly (`StoreGlobal` stores the wrapper), but the return value from calling the wrapper via `mb_call0` is incorrectly interpreted.

## Affected Files

- `crates/mamba/src/runtime/class.rs` — `mb_call0`, `mb_call1_val`
- `crates/mamba/src/lower/hir_to_mir.rs` — decorated function call dispatch (line ~3431)
- `crates/mamba/src/codegen/cranelift/jit.rs` — JIT function return convention

## Fix Direction

Align JIT return convention with dynamic dispatch: either make JIT functions return NaN-boxed `MbValue` consistently, or have `mb_call0` handle raw `i64` returns correctly.
