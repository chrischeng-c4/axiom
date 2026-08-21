# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "mean_is_callable"
# subject = "statistics.mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.mean: mean_is_callable (surface)."""
import statistics

assert callable(statistics.mean)
print("mean_is_callable OK")
