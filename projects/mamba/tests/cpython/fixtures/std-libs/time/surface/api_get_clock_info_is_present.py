# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_get_clock_info_is_present"
# subject = "time.get_clock_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.get_clock_info: api_get_clock_info_is_present (surface)."""
import time

assert hasattr(time, "get_clock_info")
print("api_get_clock_info_is_present OK")
