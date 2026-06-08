# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_quantiles_is_present"
# subject = "statistics.quantiles"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.quantiles: api_quantiles_is_present (surface)."""
import statistics

assert hasattr(statistics, "quantiles")
print("api_quantiles_is_present OK")
