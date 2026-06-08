---
number: 392
title: "feat(mamba): stdlib itertools"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #392 — feat(mamba): stdlib itertools

## Summary
Implement `itertools` standard library module.

## Required Functions
- `chain(*iterables)`, `chain.from_iterable()`
- `combinations(iterable, r)`, `combinations_with_replacement()`
- `permutations(iterable, r=None)`
- `product(*iterables, repeat=1)`
- `count(start=0, step=1)`, `cycle(iterable)`, `repeat(obj, times=None)`
- `islice(iterable, stop)`, `islice(iterable, start, stop, step)`
- `groupby(iterable, key=None)`
- `starmap(func, iterable)`
- `takewhile(pred, iterable)`, `dropwhile(pred, iterable)`
- `zip_longest(*iterables, fillvalue=None)`
- `accumulate(iterable, func=operator.add)`
- `tee(iterable, n=2)`

## Implementation Notes
- All return lazy iterators — need MbIterator wrapper objects
- Can implement in Rust for performance
