# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "variance_single_raises"
# subject = "statistics.variance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.variance: variance_single_raises (errors)."""
import statistics

_raised = False
try:
    statistics.variance([1])
except statistics.StatisticsError:
    _raised = True
assert _raised, "variance_single_raises: expected statistics.StatisticsError"
print("variance_single_raises OK")
