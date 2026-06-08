# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_iter_builtins_ops"
# subject = "cpython321.test_iter_builtins_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_iter_builtins_ops.py"
# status = "filled"
# ///
"""cpython321.test_iter_builtins_ops: execute CPython 3.12 seed test_iter_builtins_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for iterator/sequence builtins:
# enumerate, zip, map, filter, sorted, reversed, any, all, sum,
# min, max. Each verifies basic happy-path behavior on small lists.
# Companion to stub/test_builtins.py — vendored unittest seed.
_ledger: list[int] = []
xs = [10, 20, 30]
pairs = list(enumerate(xs))
assert pairs == [(0, 10), (1, 20), (2, 30)]; _ledger.append(1)
zipped = list(zip([1, 2, 3], ["a", "b", "c"]))
assert zipped == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
doubled = list(map(lambda n: n * 2, [1, 2, 3]))
assert doubled == [2, 4, 6]; _ledger.append(1)
evens = list(filter(lambda n: n % 2 == 0, [1, 2, 3, 4, 5, 6]))
assert evens == [2, 4, 6]; _ledger.append(1)
assert sorted([3, 1, 4, 1, 5, 9, 2, 6]) == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)
assert sorted([3, 1, 4], reverse=True) == [4, 3, 1]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)
assert any([False, False, True]); _ledger.append(1)
assert not any([False, False, False]); _ledger.append(1)
assert all([True, True, True]); _ledger.append(1)
assert not all([True, False, True]); _ledger.append(1)
assert sum([1, 2, 3, 4, 5]) == 15; _ledger.append(1)
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)
assert min([3, 1, 4, 1, 5]) == 1; _ledger.append(1)
assert max([3, 1, 4, 1, 5]) == 5; _ledger.append(1)
assert min(7, 2, 9) == 2; _ledger.append(1)
assert max(7, 2, 9) == 9; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_iter_builtins_ops {sum(_ledger)} asserts")
