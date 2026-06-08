# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_pvariance_is_present"
# subject = "statistics.pvariance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.pvariance: api_pvariance_is_present (surface)."""
import statistics

assert hasattr(statistics, "pvariance")
print("api_pvariance_is_present OK")
