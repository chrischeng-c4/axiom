# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "pvariance_is_callable"
# subject = "statistics.pvariance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.pvariance: pvariance_is_callable (surface)."""
import statistics

assert callable(statistics.pvariance)
print("pvariance_is_callable OK")
