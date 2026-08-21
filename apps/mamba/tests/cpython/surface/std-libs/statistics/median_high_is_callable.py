# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "median_high_is_callable"
# subject = "statistics.median_high"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.median_high: median_high_is_callable (surface)."""
import statistics

assert callable(statistics.median_high)
print("median_high_is_callable OK")
