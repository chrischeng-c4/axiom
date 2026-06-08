---
number: 1110
title: "fix(mamba): lambda limitations — nested lambdas, default arg capture, unary minus"
state: open
labels: [type:bug, priority:p1, crate:mamba]
group: "lambda-limitations"
---

# #1110 — fix(mamba): lambda limitations — nested lambdas, default arg capture, unary minus

## Problem

Lambda expressions fail in several non-trivial but common patterns:

```python
# Unary minus in lambda
sorted([3,1,2], key=lambda x: -x)  # type checker error

# Nested lambda
f = lambda x: lambda y: x + y      # fails

# Lambda with default arg capture
funcs = [lambda x=i: x for i in range(3)]  # wrong capture

# Lambda in map/filter
list(map(lambda x: x*2, [1,2,3]))  # empty result
```

## Impact

~3 conformance xfail fixtures blocked.

## Affected Files

- `crates/mamba/src/lower/hir_to_mir.rs` — lambda lowering
- `crates/mamba/src/codegen/cranelift/` — closure codegen
