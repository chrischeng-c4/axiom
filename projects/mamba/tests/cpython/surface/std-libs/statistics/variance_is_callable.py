# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "variance_is_callable"
# subject = "statistics.variance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.variance: variance_is_callable (surface)."""
import statistics

assert callable(statistics.variance)
print("variance_is_callable OK")
