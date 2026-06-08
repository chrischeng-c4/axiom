---
number: 1109
title: "fix(mamba): no-arg constructor codegen verifier error — list(), tuple(), set()"
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "mamba-codegen-runtime-fixes"
---

# #1109 — fix(mamba): no-arg constructor codegen verifier error — list(), tuple(), set()

## Problem

Calling built-in constructors with zero arguments triggers a codegen verifier error:

```python
list()    # codegen verifier error
tuple()   # codegen verifier error
set()     # codegen verifier error
```

These are among the most basic Python expressions.

## Impact

~4 conformance xfail fixtures blocked. `list()`, `tuple()`, `set()` are fundamental constructors used everywhere.

## Root Cause

The Cranelift codegen path for zero-argument builtin calls does not emit the correct function signature, causing the verifier to reject the generated IR.

## Affected Files

- `crates/mamba/src/codegen/cranelift/` — zero-arg call emission
- `crates/mamba/src/runtime/builtins.rs` — constructor dispatch
