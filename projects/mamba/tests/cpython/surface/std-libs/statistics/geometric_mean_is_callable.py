# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "geometric_mean_is_callable"
# subject = "statistics.geometric_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.geometric_mean: geometric_mean_is_callable (surface)."""
import statistics

assert callable(statistics.geometric_mean)
print("geometric_mean_is_callable OK")
