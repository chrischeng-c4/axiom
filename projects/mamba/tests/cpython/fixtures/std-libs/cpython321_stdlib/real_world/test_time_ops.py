# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_time_ops"
# subject = "cpython321.test_time_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_time_ops.py"
# status = "filled"
# ///
"""cpython321.test_time_ops: execute CPython 3.12 seed test_time_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `time` stdlib module.
# Surface: time() returns a positive epoch float, monotonic() is
# monotonically non-decreasing, sleep(0) returns without error.
# Companion to stub/test_time.py — vendored unittest seed.
import time
_ledger: list[int] = []
t1 = time.time()
assert t1 > 1000000000.0; _ledger.append(1)
assert t1 < 9999999999.0; _ledger.append(1)
m1 = time.monotonic()
m2 = time.monotonic()
assert m2 >= m1; _ledger.append(1)
time.sleep(0)
m3 = time.monotonic()
assert m3 >= m2; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_time_ops {sum(_ledger)} asserts")
