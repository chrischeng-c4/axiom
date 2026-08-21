# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "correlation_and_covariance"
# subject = "statistics.correlation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.correlation: correlation/covariance track the sign of a linear relationship (+1/-1/0/0.5), correlation is scale-invariant while covariance is not, and the 'ranked' method gives Spearman correlation"""
from statistics import correlation, covariance

# correlation and covariance track the sign/strength of a linear relationship.
for x, y, r in [([1, 2, 3], [1, 2, 3], 1),
                ([1, 2, 3], [3, 2, 1], -1),
                ([1, 2, 3], [1, 2, 1], 0),
                ([1, 2, 3], [1, 3, 2], 0.5)]:
    assert abs(correlation(x, y) - r) < 1e-7, (x, y)
    assert abs(covariance(x, y) - r) < 1e-7, (x, y)
# correlation is scale-invariant; covariance is not.
assert abs(correlation([1, 2, 3], [10, 30, 20]) - 0.5) < 1e-7
assert abs(covariance([1, 2, 3], [10, 30, 20]) - 5) < 1e-7
# The 'ranked' method gives Spearman correlation on monotonic, tie-bearing data.
reading = [56, 75, 45, 71, 61, 64, 58, 80, 76, 61]
maths = [66, 70, 40, 60, 65, 56, 59, 77, 67, 63]
assert abs(correlation(reading, maths, method="ranked") - 0.6686960980480712) < 1e-7

print("correlation_and_covariance OK")
