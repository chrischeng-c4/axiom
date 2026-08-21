# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "median_is_callable"
# subject = "statistics.median"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.median: median_is_callable (surface)."""
import statistics

assert callable(statistics.median)
print("median_is_callable OK")
