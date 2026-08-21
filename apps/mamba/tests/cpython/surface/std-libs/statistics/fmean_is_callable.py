# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "fmean_is_callable"
# subject = "statistics.fmean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.fmean: fmean_is_callable (surface)."""
import statistics

assert callable(statistics.fmean)
print("fmean_is_callable OK")
