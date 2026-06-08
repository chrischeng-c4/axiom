# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "median_low_is_callable"
# subject = "statistics.median_low"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.median_low: median_low_is_callable (surface)."""
import statistics

assert callable(statistics.median_low)
print("median_low_is_callable OK")
