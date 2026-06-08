# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_realtime_is_present"
# subject = "time.CLOCK_REALTIME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.CLOCK_REALTIME: api_clock_realtime_is_present (surface)."""
import time

assert hasattr(time, "CLOCK_REALTIME")
print("api_clock_realtime_is_present OK")
