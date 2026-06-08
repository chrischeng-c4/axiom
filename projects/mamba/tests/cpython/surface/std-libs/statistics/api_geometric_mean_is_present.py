# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_geometric_mean_is_present"
# subject = "statistics.geometric_mean"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.geometric_mean: api_geometric_mean_is_present (surface)."""
import statistics

assert hasattr(statistics, "geometric_mean")
print("api_geometric_mean_is_present OK")
