---
number: 1111
title: "fix(mamba): string reverse slice [::-1] returns empty string"
state: open
labels: [type:bug, priority:p0, crate:mamba]
group: "string-reverse-slice"
---

# #1111 — fix(mamba): string reverse slice [::-1] returns empty string

## Problem

```python
s = 'abcdef'
print(s[::-1])  # Expected: 'fedcba', Got: ''
```

Negative step slicing on strings returns an empty string instead of the reversed result.

## Impact

1 conformance xfail fixture + any real-world code using string reversal.

## Affected Files

- `crates/mamba/src/runtime/string_ops.rs` — slice with negative step
