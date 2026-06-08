# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_statistics_advanced_ops"
# subject = "cpython321.test_statistics_advanced_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_statistics_advanced_ops.py"
# status = "filled"
# ///
"""cpython321.test_statistics_advanced_ops: execute CPython 3.12 seed test_statistics_advanced_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for statistics module surfaces
# beyond test_statistics_ops (which covers mean/median/mode/variance/
# stdev/pstdev).
# Surface: pvariance (population variance, divides by N not N-1);
# harmonic_mean (reciprocal-mean); median_low / median_high (the two
# middle picks for even-length samples — odd-length samples pick the
# same single middle); quantiles (default n=4 → three cut points at
# the 25/50/75 percentiles); geometric_mean (nth root of the product);
# multimode (returns ALL modes tied for max frequency); fmean (float
# fast-path mean).
import statistics
_ledger: list[int] = []

# pvariance — divides by N (population variance)
# For [1,2,3,4,5], mean=3, sum_sq_dev = 4+1+0+1+4 = 10, pvar = 10/5 = 2.0
assert statistics.pvariance([1, 2, 3, 4, 5]) == 2.0; _ledger.append(1)
# pvariance of a uniform list is 0
assert statistics.pvariance([5, 5, 5, 5]) == 0.0; _ledger.append(1)

# harmonic_mean — reciprocal mean
# For [1, 2, 4]: 3 / (1 + 0.5 + 0.25) = 3 / 1.75
hm = statistics.harmonic_mean([1, 2, 4])
assert hm == 3.0 / 1.75; _ledger.append(1)
# harmonic_mean of identical values returns that value
assert statistics.harmonic_mean([4.0, 4.0, 4.0]) == 4.0; _ledger.append(1)

# median_low / median_high differ on even-length sets — they pick the
# lower and upper of the two middles
assert statistics.median_low([1, 2, 3, 4]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3, 4]) == 3; _ledger.append(1)
# Odd-length: both return the same single middle value
assert statistics.median_low([1, 2, 3]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3]) == 2; _ledger.append(1)
# Single-element edge case
assert statistics.median_low([42]) == 42; _ledger.append(1)
assert statistics.median_high([42]) == 42; _ledger.append(1)

# quantiles with default n=4 — produces THREE cut points at the
# 25th / 50th / 75th percentiles
q = statistics.quantiles([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
assert q == [2.75, 5.5, 8.25]; _ledger.append(1)
# Explicit n=4 yields the same as the default
q2 = statistics.quantiles([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], n=4)
assert q2 == [2.75, 5.5, 8.25]; _ledger.append(1)
# The middle cut point matches the median for the same data
assert q[1] == 5.5; _ledger.append(1)

# geometric_mean — nth root of the product
# For [1, 2, 4]: (1*2*4)^(1/3) = 8^(1/3) = 2.0
assert statistics.geometric_mean([1, 2, 4]) == 2.0; _ledger.append(1)

# multimode returns ALL modes (every value tied for max frequency)
# In [1, 1, 2, 2, 3] both 1 and 2 occur twice — both are modes
assert statistics.multimode([1, 1, 2, 2, 3]) == [1, 2]; _ledger.append(1)
# Single mode list
assert statistics.multimode([1, 1, 1, 2]) == [1]; _ledger.append(1)

# fmean — fast float-only mean
assert statistics.fmean([1, 2, 3, 4, 5]) == 3.0; _ledger.append(1)
assert statistics.fmean([2.0, 4.0]) == 3.0; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_statistics_advanced_ops {sum(_ledger)} asserts")
