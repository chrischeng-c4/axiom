# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "surface"
# case = "api_multimode_is_present"
# subject = "statistics.multimode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""statistics.multimode: api_multimode_is_present (surface)."""
import statistics

assert hasattr(statistics, "multimode")
print("api_multimode_is_present OK")
