# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_map_filter_zip_ops"
# subject = "cpython321.test_map_filter_zip_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_map_filter_zip_ops.py"
# status = "filled"
# ///
"""cpython321.test_map_filter_zip_ops: execute CPython 3.12 seed test_map_filter_zip_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for map, filter, and zip — the
# three core functional-iteration builtins.
# Surface: map(callable, iter) returns an iterator that applies the
# callable to each item; filter(callable, iter) keeps only items for
# which callable(item) is truthy; filter(None, iter) drops every
# falsy element; zip over two iterables pairs them; zip stops at
# the shortest iterable when lengths differ; zip over three
# iterables produces 3-tuples.
_ledger: list[int] = []

# map(lambda, list) applies the lambda elementwise
m = list(map(lambda x: x * 2, [1, 2, 3]))
assert m == [2, 4, 6]; _ledger.append(1)

# map accepts a builtin as the callable
ms = list(map(str, [1, 2, 3]))
assert ms == ["1", "2", "3"]; _ledger.append(1)

# map preserves length when the input is non-empty
assert len(list(map(lambda x: x, [10, 20, 30, 40]))) == 4; _ledger.append(1)

# map over an empty iterable yields an empty list
assert list(map(lambda x: x * 10, [])) == []; _ledger.append(1)

# filter(callable, list) keeps only items where callable(item) is truthy
f = list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5]))
assert f == [2, 4]; _ledger.append(1)

# filter that keeps everything
assert list(filter(lambda x: True, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# filter that keeps nothing
assert list(filter(lambda x: False, [1, 2, 3])) == []; _ledger.append(1)

# filter(None, iterable) drops every falsy element
fn = list(filter(None, [0, 1, "", "x", None, 2]))
assert fn == [1, "x", 2]; _ledger.append(1)

# filter over an empty iterable yields an empty list
assert list(filter(lambda x: True, [])) == []; _ledger.append(1)

# zip over two iterables pairs them in order
z = list(zip([1, 2, 3], ["a", "b", "c"]))
assert z == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)

# zip stops at the shortest iterable
zs = list(zip([1, 2], ["a", "b", "c"]))
assert zs == [(1, "a"), (2, "b")]; _ledger.append(1)
assert len(zs) == 2; _ledger.append(1)

# zip over three iterables produces 3-tuples
z3 = list(zip([1, 2], [3, 4], [5, 6]))
assert z3 == [(1, 3, 5), (2, 4, 6)]; _ledger.append(1)

# zip over no iterables yields an empty iterator
assert list(zip()) == []; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_map_filter_zip_ops {sum(_ledger)} asserts")
