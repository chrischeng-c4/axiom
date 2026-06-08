# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "median_grouped_is_callable"
# subject = "statistics.median_grouped"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.median_grouped: median_grouped_is_callable (surface)."""
import statistics

assert callable(statistics.median_grouped)
print("median_grouped_is_callable OK")
