# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_normal_dist_is_present"
# subject = "statistics.NormalDist"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.NormalDist: api_normal_dist_is_present (surface)."""
import statistics

assert hasattr(statistics, "NormalDist")
print("api_normal_dist_is_present OK")
