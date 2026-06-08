---
number: 387
title: "feat(mamba): for/while else clause"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #387 — feat(mamba): for/while else clause

## Summary
Implement `else` clause on `for` and `while` loops.

```python
for x in items:
    if x == target:
        break
else:
    print("not found")  # runs only if loop didn't break
```

## Required
- `for ... else:` — else block runs if loop completes without `break`
- `while ... else:` — same semantics
- Parser may already handle this (check); codegen needs a `did_break` flag variable

## Implementation Notes
- In MIR: add a boolean `did_break` variable, set to true on break, check in else block
- Or: use block structure — else block is the natural fallthrough, break jumps past it
