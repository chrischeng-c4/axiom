# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "errors"
# case = "harmonic_mean_negative_raises"
# subject = "statistics.harmonic_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_statistics.py"
# status = "filled"
# ///
"""statistics.harmonic_mean: harmonic_mean_negative_raises (errors)."""
import statistics

_raised = False
try:
    statistics.harmonic_mean([1, -2, 3])
except statistics.StatisticsError:
    _raised = True
assert _raised, "harmonic_mean_negative_raises: expected statistics.StatisticsError"
print("harmonic_mean_negative_raises OK")
