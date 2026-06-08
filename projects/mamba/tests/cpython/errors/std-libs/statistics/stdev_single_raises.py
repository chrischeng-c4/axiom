# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "stdev_single_raises"
# subject = "statistics.stdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.stdev: stdev_single_raises (errors)."""
import statistics

_raised = False
try:
    statistics.stdev([1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "stdev_single_raises: expected statistics.StatisticsError"
print("stdev_single_raises OK")
