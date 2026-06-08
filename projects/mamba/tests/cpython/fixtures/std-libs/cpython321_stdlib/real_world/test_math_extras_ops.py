# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_math_extras_ops"
# subject = "cpython321.test_math_extras_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_math_extras_ops.py"
# status = "filled"
# ///
"""cpython321.test_math_extras_ops: execute CPython 3.12 seed test_math_extras_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for math.prod/dist/hypot/isclose
# and integer-style functions (gcd, lcm, perm, comb).
# Companion to stub/test_math.py — vendored unittest seed.
import math
_ledger: list[int] = []
assert math.prod([1, 2, 3, 4]) == 24; _ledger.append(1)
assert math.prod([5]) == 5; _ledger.append(1)
assert math.prod([], start=10) == 10; _ledger.append(1)
assert math.dist((0, 0), (3, 4)) == 5.0; _ledger.append(1)
assert math.hypot(3, 4) == 5.0; _ledger.append(1)
assert math.hypot(0, 0) == 0.0; _ledger.append(1)
assert math.isclose(0.1 + 0.2, 0.3); _ledger.append(1)
assert not math.isclose(1.0, 2.0); _ledger.append(1)
assert math.gcd(48, 18) == 6; _ledger.append(1)
assert math.gcd(0, 5) == 5; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_math_extras_ops {sum(_ledger)} asserts")
