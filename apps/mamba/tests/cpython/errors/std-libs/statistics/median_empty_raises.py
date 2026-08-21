# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "median_empty_raises"
# subject = "statistics.median"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.median: median_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.median([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "median_empty_raises: expected statistics.StatisticsError"
print("median_empty_raises OK")
