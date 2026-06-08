# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mean_empty_raises"
# subject = "statistics.mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mean([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "mean_empty_raises: expected statistics.StatisticsError"
print("mean_empty_raises OK")
