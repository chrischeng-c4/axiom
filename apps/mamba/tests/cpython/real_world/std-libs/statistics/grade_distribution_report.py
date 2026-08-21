# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "real_world"
# case = "grade_distribution_report"
# subject = "statistics"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics: a grading analytics job computes mean/median/mode/stdev/variance over a synthetic score set, fits a NormalDist via from_samples to z-score a student, derives quartile cutoffs with quantiles, and correlates two assignment columns -- asserting each result against deterministic CPython values"""
import statistics
from statistics import NormalDist

# An end-of-term grading job over a fixed cohort of exam scores.
scores = [55, 62, 62, 70, 71, 74, 78, 80, 80, 80, 85, 88, 91, 95, 100]

# Central tendency and spread.
assert abs(statistics.mean(scores) - sum(scores) / len(scores)) < 1e-9
assert statistics.median(scores) == 80
assert statistics.mode(scores) == 80  # three students scored 80
_var = statistics.variance(scores)
_std = statistics.stdev(scores)
assert abs(_var - _std ** 2) < 1e-9
assert abs(_std - statistics.stdev(scores)) < 1e-12

# Fit a normal model and z-score a single student against the cohort.
model = NormalDist.from_samples(scores)
assert abs(model.mean - statistics.mean(scores)) < 1e-9
assert abs(model.stdev - _std) < 1e-9
z_top = model.zscore(100)
z_bottom = model.zscore(55)
assert z_top > 0 > z_bottom  # the 100 is above the mean, the 55 below it.

# Quartile cutoffs partition the cohort into four bands.
q1, q2, q3 = statistics.quantiles(scores, n=4)
assert q1 < q2 < q3
assert q2 == statistics.median(scores)  # the middle quartile is the median.

# Correlate two assignment columns for the same students (monotone -> positive).
homework = [50, 60, 58, 66, 70, 72, 75, 79, 81, 80, 84, 90, 89, 96, 99]
r = statistics.correlation(scores, homework)
cov = statistics.covariance(scores, homework)
assert 0.9 < r <= 1.0 and cov > 0
slope, intercept = statistics.linear_regression(scores, homework)
assert slope > 0  # higher exam scores predict higher homework scores.

print("grade_distribution_report OK")
