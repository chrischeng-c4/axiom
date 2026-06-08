# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "geometric_mean_negative_raises"
# subject = "statistics.geometric_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.geometric_mean: geometric_mean_negative_raises (errors)."""
import statistics

_raised = False
try:
    statistics.geometric_mean([1, -1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "geometric_mean_negative_raises: expected statistics.StatisticsError"
print("geometric_mean_negative_raises OK")
