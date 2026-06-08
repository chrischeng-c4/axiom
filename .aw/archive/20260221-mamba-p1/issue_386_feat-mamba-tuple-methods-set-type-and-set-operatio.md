---
number: 386
title: "feat(mamba): tuple methods, set type, and set operations"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #386 — feat(mamba): tuple methods, set type, and set operations

## Summary
Complete tuple support and implement the `set` and `frozenset` built-in types.

## Tuple
- Tuple unpacking: `a, b, c = (1, 2, 3)` (partially exists)
- Extended unpacking: `a, *rest = (1, 2, 3, 4)`
- Tuple methods: `count(value)`, `index(value)`
- Named tuples (lower priority)

## Set
- Set literal: `{1, 2, 3}`
- Set constructor: `set(iterable)`
- Methods: `add`, `remove`, `discard`, `pop`, `clear`, `copy`
- Set operations: `union (|)`, `intersection (&)`, `difference (-)`, `symmetric_difference (^)`
- `issubset`, `issuperset`, `isdisjoint`
- Set comprehensions: `{x for x in items}`
- `frozenset` (immutable variant, hashable)

## Implementation Notes
- Sets need a new `ObjData::Set(HashSet<i64>)` variant
- Set elements must be hashable (need `__hash__` protocol)
