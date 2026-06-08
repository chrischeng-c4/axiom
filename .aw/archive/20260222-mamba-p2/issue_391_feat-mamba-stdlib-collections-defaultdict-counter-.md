---
number: 391
title: "feat(mamba): stdlib collections (defaultdict, Counter, deque, OrderedDict)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #391 — feat(mamba): stdlib collections (defaultdict, Counter, deque, OrderedDict)

## Summary
Implement `collections` standard library module.

## Required Classes
- `defaultdict(default_factory)` — dict with default value factory
- `Counter(iterable)` — counting dict with `.most_common()`, arithmetic ops
- `deque(iterable, maxlen=None)` — double-ended queue with `appendleft`, `popleft`, `rotate`
- `OrderedDict` — insertion-ordered dict (mostly for compat, dict is ordered in 3.7+)
- `namedtuple(typename, field_names)` — immutable named record

## Implementation Notes
- Each needs a new `ObjData` variant or class-based implementation
- `Counter` can be built on top of dict
- `deque` needs `VecDeque<i64>` backend
