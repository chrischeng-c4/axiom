# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "mean_nonnumeric_raises"
# subject = "statistics.mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.mean: mean_nonnumeric_raises (errors)."""
import statistics

_raised = False
try:
    statistics.mean([1, 2, '3'])
except TypeError:
    _raised = True
assert _raised, "mean_nonnumeric_raises: expected TypeError"
print("mean_nonnumeric_raises OK")
