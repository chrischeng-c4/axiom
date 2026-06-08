# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_mode_is_present"
# subject = "statistics.mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.mode: api_mode_is_present (surface)."""
import statistics

assert hasattr(statistics, "mode")
print("api_mode_is_present OK")
