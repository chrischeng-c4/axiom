# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_pstdev_is_present"
# subject = "statistics.pstdev"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.pstdev: api_pstdev_is_present (surface)."""
import statistics

assert hasattr(statistics, "pstdev")
print("api_pstdev_is_present OK")
