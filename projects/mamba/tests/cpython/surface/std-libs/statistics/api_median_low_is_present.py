# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_median_low_is_present"
# subject = "statistics.median_low"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.median_low: api_median_low_is_present (surface)."""
import statistics

assert hasattr(statistics, "median_low")
print("api_median_low_is_present OK")
