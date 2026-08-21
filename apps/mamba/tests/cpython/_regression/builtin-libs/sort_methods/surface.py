# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""builtin-libs/sort_methods: surface probes (CPython 3.12 oracle)."""

# Probes for the documented sort-related builtin surface.

# list.sort exists and is callable; sorted/min/max are builtins.
assert callable(list.sort)
assert callable(sorted)
assert callable(min)
assert callable(max)

# functools.cmp_to_key is the documented bridge for old-style comparators.
from functools import cmp_to_key
assert callable(cmp_to_key)

# sorted() accepts the keyword-only `key` and `reverse` parameters.
assert sorted([3, 1, 2], key=None, reverse=False) == [1, 2, 3]

# list.sort() accepts the same keyword-only parameters and returns None.
probe = [3, 1, 2]
assert probe.sort(key=None, reverse=True) is None
assert probe == [3, 2, 1]

# min/max accept a `key` callable and a `default` for empty iterables.
assert min(["aa", "b", "ccc"], key=len) == "b"
assert max(["aa", "b", "ccc"], key=len) == "ccc"
assert min([], default="none") == "none"

print("surface OK")
