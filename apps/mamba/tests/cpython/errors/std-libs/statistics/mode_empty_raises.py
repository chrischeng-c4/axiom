# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mode_empty_raises"
# subject = "statistics.mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mode: mode_empty_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mode([])
except statistics.StatisticsError:
    _raised = True
assert _raised, "mode_empty_raises: expected statistics.StatisticsError"
print("mode_empty_raises OK")
