# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_itertools_groupby_ops"
# subject = "cpython321.test_itertools_groupby_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_itertools_groupby_ops.py"
# status = "filled"
# ///
"""cpython321.test_itertools_groupby_ops: execute CPython 3.12 seed test_itertools_groupby_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `itertools.groupby` — a
# surface only lightly touched by test_itertools.py (one stub-style
# call) and absent from the dedicated itertools ops fixtures. This
# seed asserts: groupby returns an iterator of (key, group) pairs
# where the group is itself an iterator of consecutive equal
# elements; consecutive equal int / str / bool runs collapse to a
# single (key, run) pair; non-consecutive duplicates produce
# separate (key, run) pairs (groupby is NOT global-deduplication);
# a singleton run collapses to a one-element group; the empty
# iterable produces an empty group sequence; groupby over a
# str / list / tuple all preserve their element-equality behaviour.
import itertools
_ledger: list[int] = []

# Consecutive equal runs of ints
g1 = [(k, list(g)) for k, g in itertools.groupby([1, 1, 2, 2, 3])]
assert g1 == [(1, [1, 1]), (2, [2, 2]), (3, [3])]; _ledger.append(1)

g2 = [(k, list(g)) for k, g in itertools.groupby([5, 5, 5])]
assert g2 == [(5, [5, 5, 5])]; _ledger.append(1)

g3 = [(k, list(g)) for k, g in itertools.groupby([1, 2, 3])]
assert g3 == [(1, [1]), (2, [2]), (3, [3])]; _ledger.append(1)

# Non-consecutive duplicates produce separate groups
g4 = [(k, list(g)) for k, g in itertools.groupby([1, 2, 1, 2, 1])]
assert g4 == [(1, [1]), (2, [2]), (1, [1]), (2, [2]), (1, [1])]; _ledger.append(1)

g5 = [(k, list(g)) for k, g in itertools.groupby([1, 1, 2, 1, 1])]
assert g5 == [(1, [1, 1]), (2, [2]), (1, [1, 1])]; _ledger.append(1)

# Singleton input
g6 = [(k, list(g)) for k, g in itertools.groupby([42])]
assert g6 == [(42, [42])]; _ledger.append(1)

# Empty input
g7 = [(k, list(g)) for k, g in itertools.groupby([])]
assert g7 == []; _ledger.append(1)

# Strings — groupby treats each char as element
g8 = [(k, list(g)) for k, g in itertools.groupby("aabbbc")]
assert g8 == [("a", ["a", "a"]), ("b", ["b", "b", "b"]), ("c", ["c"])]; _ledger.append(1)

g9 = [(k, list(g)) for k, g in itertools.groupby("aaa")]
assert g9 == [("a", ["a", "a", "a"])]; _ledger.append(1)

g10 = [(k, list(g)) for k, g in itertools.groupby("abc")]
assert g10 == [("a", ["a"]), ("b", ["b"]), ("c", ["c"])]; _ledger.append(1)

g11 = [(k, list(g)) for k, g in itertools.groupby("")]
assert g11 == []; _ledger.append(1)

# Tuple input — preserves element equality
g12 = [(k, list(g)) for k, g in itertools.groupby((1, 1, 2, 3, 3))]
assert g12 == [(1, [1, 1]), (2, [2]), (3, [3, 3])]; _ledger.append(1)

# Mixed-value runs — booleans on their own (treat as separate from ints
# for this seed by using a pure-bool iterable)
g13 = [(k, list(g)) for k, g in itertools.groupby([True, True, False, False, True])]
assert g13 == [(True, [True, True]), (False, [False, False]), (True, [True])]; _ledger.append(1)

# Run-length representation use case
def run_lengths(seq):
    return [(k, len(list(g))) for k, g in itertools.groupby(seq)]

assert run_lengths("aaabbc") == [("a", 3), ("b", 2), ("c", 1)]; _ledger.append(1)
assert run_lengths([1, 1, 1, 2, 3, 3]) == [(1, 3), (2, 1), (3, 2)]; _ledger.append(1)
assert run_lengths([]) == []; _ledger.append(1)
assert run_lengths([7]) == [(7, 1)]; _ledger.append(1)

# Keys emitted from groupby — extract just keys
keys = [k for k, _ in itertools.groupby([1, 1, 2, 3, 3, 1])]
assert keys == [1, 2, 3, 1]; _ledger.append(1)
keys2 = [k for k, _ in itertools.groupby("aabbcca")]
assert keys2 == ["a", "b", "c", "a"]; _ledger.append(1)

# Count groups
gcount = sum(1 for _ in itertools.groupby([1, 1, 2, 2, 3, 3]))
assert gcount == 3; _ledger.append(1)
gcount2 = sum(1 for _ in itertools.groupby([1, 2, 3, 4]))
assert gcount2 == 4; _ledger.append(1)
gcount3 = sum(1 for _ in itertools.groupby([1, 1, 1, 1]))
assert gcount3 == 1; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_itertools_groupby_ops {sum(_ledger)} asserts")
