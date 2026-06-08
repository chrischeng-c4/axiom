# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "linear_regression_is_callable"
# subject = "statistics.linear_regression"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.linear_regression: linear_regression_is_callable (surface)."""
import statistics

assert callable(statistics.linear_regression)
print("linear_regression_is_callable OK")
