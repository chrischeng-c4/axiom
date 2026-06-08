# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "bivariate_reject_unequal_and_small_inputs"
# subject = "statistics.linear_regression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.linear_regression: covariance/correlation/linear_regression all raise StatisticsError on unequal-length or fewer-than-two-point inputs, constant x cannot be regressed, and an unknown correlation method is a ValueError"""
from statistics import correlation, covariance, linear_regression, StatisticsError

# All three reject unequal-length and fewer-than-two-point inputs.
for x, y in [([1, 2, 3], [1, 2]), ([1], [1]), ([], [])]:
    for fn in (covariance, correlation, linear_regression):
        _raised = False
        try:
            fn(x, y)
        except StatisticsError:
            _raised = True
        assert _raised, (fn.__name__, x, y)

# Constant x cannot be regressed.
_raised = False
try:
    linear_regression([1, 1, 1], [1, 2, 3])
except StatisticsError:
    _raised = True
assert _raised, "constant x should raise StatisticsError"

# An unknown correlation method is a ValueError.
_raised = False
try:
    correlation([1, 2, 3], [1, 3, 2], method="bogus")
except ValueError:
    _raised = True
assert _raised, "unknown correlation method should raise ValueError"

print("bivariate_reject_unequal_and_small_inputs OK")
