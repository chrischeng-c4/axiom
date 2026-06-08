# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_bisect_ops"
# subject = "cpython321.test_bisect_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_bisect_ops.py"
# status = "filled"
# ///
"""cpython321.test_bisect_ops: execute CPython 3.12 seed test_bisect_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `bisect` stdlib module.
# Surface: bisect_left/right, bisect alias, insort maintains order.
# Companion to stub/test_bisect.py — vendored unittest seed.
import bisect
_ledger: list[int] = []
xs = [1, 3, 5, 7, 9]
assert bisect.bisect_left(xs, 5) == 2; _ledger.append(1)
assert bisect.bisect_right(xs, 5) == 3; _ledger.append(1)
assert bisect.bisect(xs, 6) == 3; _ledger.append(1)
assert bisect.bisect_left(xs, 0) == 0; _ledger.append(1)
assert bisect.bisect_right(xs, 10) == 5; _ledger.append(1)
ys = [1, 3, 5, 7, 9]
bisect.insort(ys, 4)
assert ys == [1, 3, 4, 5, 7, 9]; _ledger.append(1)
zs = [10, 20, 30]
bisect.insort(zs, 25)
assert zs == [10, 20, 25, 30]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_bisect_ops {sum(_ledger)} asserts")
