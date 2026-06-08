# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_statistics_ops"
# subject = "cpython321.test_statistics_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_statistics_ops.py"
# status = "filled"
# ///
"""cpython321.test_statistics_ops: execute CPython 3.12 seed test_statistics_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `statistics` descriptive stats.
# Surface: mean / median (odd + even length) / mode / variance /
# stdev / pstdev with canonical-vector values.
# Companion to stub/test_statistics.py — vendored unittest seed.
import statistics
import math
_ledger: list[int] = []
assert statistics.mean([1, 2, 3, 4, 5]) == 3.0; _ledger.append(1)
assert statistics.mean([10, 20, 30]) == 20.0; _ledger.append(1)
# median — odd length picks middle element
assert statistics.median([1, 2, 3, 4, 5]) == 3; _ledger.append(1)
# median — even length averages the two middle elements
assert statistics.median([1, 2, 3, 4]) == 2.5; _ledger.append(1)
# mode — most common value (deterministic for unique mode)
assert statistics.mode([1, 1, 2, 3]) == 1; _ledger.append(1)
assert statistics.mode([4, 4, 4, 2, 2, 1]) == 4; _ledger.append(1)
# sample variance — uses n-1 denominator
assert statistics.variance([1, 2, 3, 4, 5]) == 2.5; _ledger.append(1)
# pstdev([1..5]) = sqrt(pvar=2) = sqrt(2)
assert math.isclose(statistics.pstdev([1, 2, 3, 4, 5]), math.sqrt(2)); _ledger.append(1)
# stdev canonical vector
assert math.isclose(statistics.stdev([2, 4, 4, 4, 5, 5, 7, 9]), 2.138089935299395); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_statistics_ops {sum(_ledger)} asserts")
