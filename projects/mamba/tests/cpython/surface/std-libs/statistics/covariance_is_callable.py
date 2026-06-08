# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "covariance_is_callable"
# subject = "statistics.covariance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.covariance: covariance_is_callable (surface)."""
import statistics

assert callable(statistics.covariance)
print("covariance_is_callable OK")
