---
number: 1113
title: "fix(mamba): iter(callable, sentinel) two-argument form not implemented"
state: open
labels: [type:bug, priority:p1, crate:mamba]
group: "iter-two-arg"
---

# #1113 — fix(mamba): iter(callable, sentinel) two-argument form not implemented

## Problem

```python
vals = iter([3, 2, 1, 0])
print(list(iter(lambda: next(vals), 0)))
# Expected: [3, 2, 1]
# Got: codegen error
```

The two-argument form of `iter(callable, sentinel)` — which calls `callable()` repeatedly until it returns `sentinel` — is not implemented.

## Impact

1 conformance xfail fixture blocked.

## Affected Files

- `crates/mamba/src/runtime/iter.rs` — `mb_iter_new` needs sentinel variant
- `crates/mamba/src/runtime/builtins.rs` — `iter()` builtin dispatch
