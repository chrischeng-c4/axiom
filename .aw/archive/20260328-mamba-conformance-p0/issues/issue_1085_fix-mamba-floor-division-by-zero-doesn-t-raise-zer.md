---
number: 1085
title: "fix(mamba): floor division by zero doesn't raise ZeroDivisionError"
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "mamba-runtime-bugs"
---

# #1085 — fix(mamba): floor division by zero doesn't raise ZeroDivisionError

## Problem

`1 // 0` silently returns 0 instead of raising `ZeroDivisionError`.

```python
try:
    x = 1 // 0
except ZeroDivisionError:
    print("caught")
# Expected: "caught", Got: (nothing)
```

## Root Cause

`mb_floordiv()` in `runtime/builtins.rs` returns `MbValue::none()` on zero divisor instead of raising `ZeroDivisionError`. The Cranelift codegen for `FloorDiv` also doesn't emit a zero-check branch before `sdiv`.

## Affected Files

- `crates/mamba/src/runtime/builtins.rs` — `mb_floordiv()` (~line 1497)
- `crates/mamba/src/codegen/cranelift/mod.rs` — `FloorDiv` codegen path

## Fix Direction

1. In `mb_floordiv()`: call `mb_raise("ZeroDivisionError", "integer division or modulo by zero")` when divisor is 0
2. In codegen: emit zero-check branch before `sdiv` instruction, branch to exception raise on zero
