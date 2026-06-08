# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "correlation_is_callable"
# subject = "statistics.correlation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.correlation: correlation_is_callable (surface)."""
import statistics

assert callable(statistics.correlation)
print("correlation_is_callable OK")
