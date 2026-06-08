# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_statistics_correlation_normaldist_ops"
# subject = "cpython321.test_statistics_correlation_normaldist_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_statistics_correlation_normaldist_ops.py"
# status = "filled"
# ///
"""cpython321.test_statistics_correlation_normaldist_ops: execute CPython 3.12 seed test_statistics_correlation_normaldist_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `statistics.correlation`,
# `statistics.covariance`, `statistics.linear_regression`, and
# `statistics.NormalDist(mu, sigma)` construction. Surface:
# correlation returns +1.0 for identically-ordered or
# proportionally-scaled vectors, -1.0 for reverse-ordered pairs;
# covariance returns the sample covariance using the conventional
# (N-1) divisor — covariance([1..n], [1..n]) returns the sample
# variance of [1..n]; linear_regression returns a 2-tuple
# `(slope, intercept)` for the least-squares fit (on mamba's runtime
# this is tuple-equality, not named-attribute access — `.slope`
# / `.intercept` named-attribute access is NOT asserted here);
# `NormalDist(mu=0, sigma=1)` exposes `.mean` and `.stdev` attributes
# carrying the standard-normal construction parameters. Non-default
# kwargs and the `.cdf()` method are NOT asserted — those subsurfaces
# are tracked separately.
import statistics
_ledger: list[int] = []

# correlation: identically-ordered vectors -> +1.0
c1 = statistics.correlation([1, 2, 3], [1, 2, 3])
assert c1 == 1.0; _ledger.append(1)
# reverse-ordered pair -> -1.0
c2 = statistics.correlation([1, 2, 3], [3, 2, 1])
assert c2 == -1.0; _ledger.append(1)
# proportionally-scaled vector still correlates +1.0
c3 = statistics.correlation([1, 2, 3, 4, 5], [2, 4, 6, 8, 10])
assert c3 == 1.0; _ledger.append(1)

# covariance: same vector with itself == sample variance
v1 = statistics.covariance([1, 2, 3], [1, 2, 3])
assert v1 == 1.0; _ledger.append(1)
# Four-element identical vector pair
v2 = statistics.covariance([1, 2, 3, 4], [1, 2, 3, 4])
assert abs(v2 - 1.6666666666666667) < 1e-9; _ledger.append(1)

# linear_regression returns (slope, intercept) tuple — y = 2x
lr = statistics.linear_regression([1, 2, 3, 4], [2, 4, 6, 8])
assert lr == (2.0, 0.0); _ledger.append(1)

# NormalDist construction exposes mean and stdev
nd = statistics.NormalDist(mu=0, sigma=1)
assert nd.mean == 0.0; _ledger.append(1)
assert nd.stdev == 1.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_statistics_correlation_normaldist_ops {sum(_ledger)} asserts")
