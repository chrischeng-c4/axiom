---
number: 1108
title: "fix(mamba): keyword argument dispatch not implemented — sorted(key=), print(sep=), etc."
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "mamba-kwargs-dispatch"
---

# #1108 — fix(mamba): keyword argument dispatch not implemented — sorted(key=), print(sep=), etc.

## Problem

Keyword argument dispatch is the single largest systemic gap in Mamba. Any builtin or function call using keyword arguments silently fails or produces wrong results.

## Affected Cases

- `sorted(key=len)`, `sorted(reverse=True)`
- `print(sep='-', end='')` 
- `min(key=...)`, `max(default=...)`
- `int('ff', base=16)`
- `pow(2, 10, mod=3)`
- `str.format(name='Bob', age=25)`
- `list.sort(key=fn, reverse=True)`
- User-defined functions with keyword-only args

## Impact

~8 conformance xfail fixtures blocked by this. Fixing this alone would raise pass rate by ~5%.

## Root Cause

The calling convention in `mb_call*` functions and Cranelift codegen does not propagate keyword arguments to callees.

## Affected Files

- `crates/mamba/src/runtime/builtins.rs` — builtin function signatures
- `crates/mamba/src/codegen/cranelift/` — call instruction emission
- `crates/mamba/src/lower/hir_to_mir.rs` — keyword arg lowering
