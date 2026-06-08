# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_variance_is_present"
# subject = "statistics.variance"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.variance: api_variance_is_present (surface)."""
import statistics

assert hasattr(statistics, "variance")
print("api_variance_is_present OK")
