---
number: 393
title: "feat(mamba): stdlib functools (partial, lru_cache, reduce)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #393 — feat(mamba): stdlib functools (partial, lru_cache, reduce)

## Summary
Implement `functools` standard library module.

## Required
- `partial(func, *args, **kwargs)` — partial function application
- `reduce(func, iterable, initial=None)` — left fold
- `lru_cache(maxsize=128)` — memoization decorator
- `cache` — unbounded cache (alias for `lru_cache(maxsize=None)`)
- `wraps(wrapped)` — decorator helper for preserving function metadata
- `cmp_to_key(func)` — convert comparison function to key function
- `total_ordering` — class decorator to fill in comparison methods

## Implementation Notes
- `partial` needs a callable wrapper MbObject that prepends captured args
- `lru_cache` needs a HashMap-based cache keyed on arguments
