# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "quantiles_is_callable"
# subject = "statistics.quantiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""statistics.quantiles: quantiles_is_callable (surface)."""
import statistics

assert callable(statistics.quantiles)
print("quantiles_is_callable OK")
