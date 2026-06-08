# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_covariance_is_present"
# subject = "statistics.covariance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.covariance: api_covariance_is_present (surface)."""
import statistics

assert hasattr(statistics, "covariance")
print("api_covariance_is_present OK")
