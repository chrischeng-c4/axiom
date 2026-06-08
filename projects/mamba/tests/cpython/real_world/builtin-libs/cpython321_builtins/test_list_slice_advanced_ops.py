# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_list_slice_advanced_ops"
# subject = "cpython321.test_list_slice_advanced_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_list_slice_advanced_ops.py"
# status = "filled"
# ///
"""cpython321.test_list_slice_advanced_ops: execute CPython 3.12 seed test_list_slice_advanced_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for list slice / copy / generator-
# extend surfaces not in test_list_ops.
# Surface: reverse-step slice `[::-1]`, slice assignment that replaces
# a span with a different-length list, pop(index), .copy() shallow
# copy, .extend() consuming a generator expression.
_ledger: list[int] = []

# Reverse-step slice — yields a reversed copy
src = [1, 2, 3, 4, 5]
assert src[::-1] == [5, 4, 3, 2, 1]; _ledger.append(1)
# Step-slice with non-trivial start/stop
assert [10, 20, 30, 40, 50, 60][1:5:2] == [20, 40]; _ledger.append(1)

# Slice assignment replaces a span with a different-length list
sa = [1, 2, 3, 4, 5]
sa[1:3] = [10, 20, 30]
assert sa == [1, 10, 20, 30, 4, 5]; _ledger.append(1)

# Slice assignment that shortens the list
sb = [1, 2, 3, 4, 5]
sb[1:4] = [99]
assert sb == [1, 99, 5]; _ledger.append(1)

# pop(index) removes and returns the element at that index
sp = [10, 20, 30, 40]
v = sp.pop(1)
assert v == 20; _ledger.append(1)
assert sp == [10, 30, 40]; _ledger.append(1)

# .copy() returns a shallow copy that is independent of the original
sc_orig = [1, 2, 3]
sc = sc_orig.copy()
sc.append(99)
assert sc_orig == [1, 2, 3]; _ledger.append(1)
assert sc == [1, 2, 3, 99]; _ledger.append(1)

# .extend() consumes a generator expression, not just a sequence
se = [1, 2]
se.extend(x * 10 for x in [3, 4, 5])
assert se == [1, 2, 30, 40, 50]; _ledger.append(1)

# Negative-step subrange — slice with stop and negative step
sn = [0, 1, 2, 3, 4, 5][4:0:-1]
assert sn == [4, 3, 2, 1]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_list_slice_advanced_ops {sum(_ledger)} asserts")
