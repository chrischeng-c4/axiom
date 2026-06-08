# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_range_indexing_slicing_ops"
# subject = "cpython321.test_range_indexing_slicing_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_range_indexing_slicing_ops.py"
# status = "filled"
# ///
"""cpython321.test_range_indexing_slicing_ops: execute CPython 3.12 seed test_range_indexing_slicing_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for range indexing, slicing, counting,
# and membership. Surface: `range(stop)` and `range(start, stop, step)`
# both materialise into a value-typed lazy sequence that supports
# constant-time indexing (`r[0]`, `r[-1]`, `r[k]`), boolean
# `in`/`not in` membership testing, and `.count(x)` / `.index(x)`
# lookup. Slicing a range produces another range (or list-coerced
# sequence) preserving stride semantics. Equality between two ranges
# compares the materialised value sequence, so `range(0, 10, 2) ==
# range(0, 10, 2)` is True regardless of identity. `reversed(range)`
# yields elements in reverse order, `enumerate(range)` pairs each
# element with its index, and `sum(range)` aggregates the elements
# (constant time in CPython, value-correct in mamba). An empty range
# (`range(0)`, `range(5, 5)`) materialises to `[]` with length 0.
# Negative-step ranges descend toward but never include `stop`.
_ledger: list[int] = []

# Materialisation
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert len(range(5)) == 5; _ledger.append(1)

# Constant-time indexing
r = range(5)
assert r[0] == 0; _ledger.append(1)
assert r[-1] == 4; _ledger.append(1)
assert r[2] == 2; _ledger.append(1)

# Three-arg start/stop materialisation
assert list(range(2, 8)) == [2, 3, 4, 5, 6, 7]; _ledger.append(1)

# Step argument materialisation
assert list(range(0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)

# Negative step descends
assert list(range(10, 0, -1)) == [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]; _ledger.append(1)

# Empty ranges
assert list(range(0)) == []; _ledger.append(1)
assert len(range(0)) == 0; _ledger.append(1)
assert list(range(5, 5)) == []; _ledger.append(1)
assert len(range(5, 5)) == 0; _ledger.append(1)

# Membership
r7 = range(10)
assert 5 in r7; _ledger.append(1)
assert 10 not in r7; _ledger.append(1)
assert -1 not in r7; _ledger.append(1)

# count / index
assert r7.count(5) == 1; _ledger.append(1)
assert r7.count(99) == 0; _ledger.append(1)
assert r7.index(5) == 5; _ledger.append(1)

# Slicing
assert list(r7[2:5]) == [2, 3, 4]; _ledger.append(1)
assert list(r7[::2]) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(r7[::-1]) == [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]; _ledger.append(1)

# Equality by value sequence
assert range(5) == range(5); _ledger.append(1)
assert range(5) != range(6); _ledger.append(1)
assert range(0, 10, 2) == range(0, 10, 2); _ledger.append(1)

# reversed / enumerate / sum
assert list(reversed(range(5))) == [4, 3, 2, 1, 0]; _ledger.append(1)
assert list(enumerate(range(3))) == [(0, 0), (1, 1), (2, 2)]; _ledger.append(1)
assert sum(range(11)) == 55; _ledger.append(1)
assert sum(range(1, 6)) == 15; _ledger.append(1)

# int-float cross-type membership
assert 3.0 in range(10); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_range_indexing_slicing_ops {sum(_ledger)} asserts")
