# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_stdev_is_present"
# subject = "statistics.stdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.stdev: api_stdev_is_present (surface)."""
import statistics

assert hasattr(statistics, "stdev")
print("api_stdev_is_present OK")
