# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "quantiles_zero_n_raises"
# subject = "statistics.quantiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.quantiles: quantiles_zero_n_raises (errors)."""
import statistics

_raised = False
try:
    statistics.quantiles([1, 2, 3], n=0)
except statistics.StatisticsError:
    _raised = True
assert _raised, "quantiles_zero_n_raises: expected statistics.StatisticsError"
print("quantiles_zero_n_raises OK")
