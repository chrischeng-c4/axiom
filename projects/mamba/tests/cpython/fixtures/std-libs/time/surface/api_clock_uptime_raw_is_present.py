# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_uptime_raw_is_present"
# subject = "time.CLOCK_UPTIME_RAW"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.CLOCK_UPTIME_RAW: api_clock_uptime_raw_is_present (surface)."""
import time

assert hasattr(time, "CLOCK_UPTIME_RAW")
print("api_clock_uptime_raw_is_present OK")
