# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_gettime_is_present"
# subject = "time.clock_gettime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.clock_gettime: api_clock_gettime_is_present (surface)."""
import time

assert hasattr(time, "clock_gettime")
print("api_clock_gettime_is_present OK")
