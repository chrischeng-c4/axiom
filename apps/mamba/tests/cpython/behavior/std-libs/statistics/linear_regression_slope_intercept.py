# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "behavior"
# case = "linear_regression_slope_intercept"
# subject = "statistics.linear_regression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.linear_regression: linear_regression returns (slope, intercept) recovering known lines, and proportional=True forces a zero intercept (line through the origin)"""
from statistics import linear_regression

# linear_regression returns (slope, intercept), recovering known lines.
for x, y, intercept, slope in [([1, 2, 3], [12, 14, 16], 10, 2),
                               ([1, 2, 3], [1, 2, 3], 0, 1),
                               ([1, 2, 3], [100, 100, 100], 100, 0)]:
    s, b = linear_regression(x, y)
    assert abs(b - intercept) < 1e-7 and abs(s - slope) < 1e-7, (x, y)
# proportional=True forces a zero intercept (the line passes through the origin).
s, b = linear_regression([10, 20, 30, 40], [180, 398, 610, 799], proportional=True)
assert b == 0.0 and abs(s - (20 + 1 / 150)) < 1e-7

print("linear_regression_slope_intercept OK")
