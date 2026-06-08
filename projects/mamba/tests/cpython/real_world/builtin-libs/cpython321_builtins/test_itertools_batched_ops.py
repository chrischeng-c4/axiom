# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_itertools_batched_ops"
# subject = "cpython321.test_itertools_batched_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_itertools_batched_ops.py"
# status = "filled"
# ///
"""cpython321.test_itertools_batched_ops: execute CPython 3.12 seed test_itertools_batched_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `itertools.batched` — the Python
# 3.12+ batching primitive that splits an iterable into fixed-size
# tuples (last tuple may be short). Existing itertools seeds
# (test_itertools, test_itertools_ops, test_itertools_chains_ops,
# test_itertools_combinatoric_ops, test_itertools_combinatoric_extras_ops,
# test_itertools_predicates_ops) cover chain / tee / repeat / accumulate /
# product / permutations / combinations / takewhile / dropwhile / starmap /
# islice / pairwise — but skip `batched` entirely. mamba 0.3.60 supports
# every probed form below.
#
# Surface:
#   • batched(seq, n) — yields tuples of size n; final tuple may be short;
#   • inputs: lists, tuples, strs (per-char), bytes (per-int), ranges,
#     iterators (one-shot), tuples-of-tuples (elements stay grouped);
#   • size = 1 yields singleton tuples; size larger than len yields
#     a single short tuple; empty input yields nothing;
#   • n = 0 / negative is a ValueError per CPython contract — verified
#     on the negative side via try/except (batched is one of the rare
#     itertools functions where mamba's argument-validation matches
#     CPython exactly).
from itertools import batched
_ledger: list[int] = []

# Exact-division batches — every chunk has size n
assert list(batched([1, 2, 3, 4], 2)) == [(1, 2), (3, 4)]; _ledger.append(1)
assert list(batched([1, 2, 3, 4, 5, 6], 3)) == [(1, 2, 3), (4, 5, 6)]; _ledger.append(1)
assert list(batched([1, 2, 3, 4, 5, 6], 6)) == [(1, 2, 3, 4, 5, 6)]; _ledger.append(1)

# Leftover — final tuple is shorter than n
assert list(batched([1, 2, 3, 4, 5], 2)) == [(1, 2), (3, 4), (5,)]; _ledger.append(1)
assert list(batched([1, 2, 3, 4, 5, 6, 7], 3)) == [(1, 2, 3), (4, 5, 6), (7,)]; _ledger.append(1)
assert list(batched(range(10), 3)) == [(0, 1, 2), (3, 4, 5), (6, 7, 8), (9,)]; _ledger.append(1)

# size = 1 — singleton tuples
assert list(batched([1, 2, 3], 1)) == [(1,), (2,), (3,)]; _ledger.append(1)
assert list(batched("abc", 1)) == [("a",), ("b",), ("c",)]; _ledger.append(1)

# size larger than len — single short tuple
assert list(batched([1, 2], 5)) == [(1, 2)]; _ledger.append(1)
assert list(batched([1], 10)) == [(1,)]; _ledger.append(1)

# Empty input — empty result regardless of n
assert list(batched([], 1)) == []; _ledger.append(1)
assert list(batched([], 5)) == []; _ledger.append(1)
assert list(batched("", 3)) == []; _ledger.append(1)

# Str input — yields tuples of single characters
assert list(batched("abcdef", 2)) == [("a", "b"), ("c", "d"), ("e", "f")]; _ledger.append(1)
assert list(batched("hello world", 4)) == [
    ("h", "e", "l", "l"),
    ("o", " ", "w", "o"),
    ("r", "l", "d"),
]; _ledger.append(1)

# Bytes input — yields tuples of int code points
assert list(batched(b"abcdef", 2)) == [(97, 98), (99, 100), (101, 102)]; _ledger.append(1)

# Tuple-of-tuples element preservation
assert list(batched([(1, 2), (3, 4), (5, 6)], 2)) == [((1, 2), (3, 4)), ((5, 6),)]; _ledger.append(1)

# Iterator input — one-shot consumption
assert list(batched(iter([1, 2, 3, 4, 5]), 2)) == [(1, 2), (3, 4), (5,)]; _ledger.append(1)

# Capture-then-consume — list() returns the full batch sequence
b = batched([1, 2, 3, 4], 2)
assert list(b) == [(1, 2), (3, 4)]; _ledger.append(1)

# n = 0 — ValueError per CPython contract; mamba matches exactly
try:
    _ = list(batched([1, 2, 3], 0))
    raise AssertionError("batched(_, 0) must raise ValueError")
except ValueError:
    _ledger.append(1)

# negative n — ValueError per CPython contract
try:
    _ = list(batched([1, 2, 3], -1))
    raise AssertionError("batched(_, -1) must raise ValueError")
except ValueError:
    _ledger.append(1)

# Tuple-of-batched is itself a tuple (not a list-wrapped form)
batches = list(batched([1, 2, 3, 4], 2))
assert isinstance(batches[0], tuple); _ledger.append(1)
assert isinstance(batches[-1], tuple); _ledger.append(1)

# n equals len exactly — single full tuple, no short tail
assert list(batched([10, 20, 30], 3)) == [(10, 20, 30)]; _ledger.append(1)

# Mixed-type elements stay as-is inside each tuple
assert list(batched([1, "a", 2.5, None], 2)) == [(1, "a"), (2.5, None)]; _ledger.append(1)

# Set input — order is implementation-defined, so check by total content
# size and tuple-shape rather than ordering.
batched_set = list(batched({1, 2, 3, 4, 5}, 2))
flat = [x for tup in batched_set for x in tup]
assert sorted(flat) == [1, 2, 3, 4, 5]; _ledger.append(1)
assert all(len(tup) <= 2 for tup in batched_set); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_itertools_batched_ops {sum(_ledger)} asserts")
