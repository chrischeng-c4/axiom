---
number: 402
title: "feat(mamba): dict/list unpacking expressions ({**d1, **d2}, [*a, *b])"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #402 — feat(mamba): dict/list unpacking expressions ({**d1, **d2}, [*a, *b])

## Summary
Implement unpacking/spread operators in dict and list literals.

## Required
- Dict unpacking: `{**d1, **d2, 'key': val}`
- List unpacking: `[*a, 1, 2, *b]`
- Tuple unpacking: `(*a, 1, *b)`
- Set unpacking: `{*a, *b}`
- Function call unpacking: `f(*args, **kwargs)` (may partially exist)
- Assignment unpacking: `a, *rest, b = [1, 2, 3, 4, 5]`

## Implementation Notes
- Parser already handles starred expressions
- Codegen needs to emit merge/extend operations for unpacking in literals
- Extended unpacking assignment (`*rest`) needs runtime support for splitting sequences
