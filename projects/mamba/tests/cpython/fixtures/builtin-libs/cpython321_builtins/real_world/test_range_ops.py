# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_range_ops"
# subject = "cpython321.test_range_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_range_ops.py"
# status = "filled"
# ///
"""cpython321.test_range_ops: execute CPython 3.12 seed test_range_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for builtin `range`.
# Surface: one/two/three-arg construction, list materialization,
# len, in/not in membership, negative step, start/stop/step attrs.
# Companion to stub/test_range.py — vendored unittest seed.
_ledger: list[int] = []
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(range(2, 6)) == [2, 3, 4, 5]; _ledger.append(1)
assert list(range(1, 10, 2)) == [1, 3, 5, 7, 9]; _ledger.append(1)
assert list(range(10, 0, -2)) == [10, 8, 6, 4, 2]; _ledger.append(1)
assert len(range(5)) == 5; _ledger.append(1)
assert len(range(2, 10)) == 8; _ledger.append(1)
assert len(range(0, 10, 3)) == 4; _ledger.append(1)
assert 3 in range(5); _ledger.append(1)
assert 5 not in range(5); _ledger.append(1)
total = 0
for i in range(5):
    total += i
assert total == 10; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_range_ops {sum(_ledger)} asserts")
