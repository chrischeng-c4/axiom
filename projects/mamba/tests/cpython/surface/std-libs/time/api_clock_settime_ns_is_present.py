# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_settime_ns_is_present"
# subject = "time.clock_settime_ns"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.clock_settime_ns: api_clock_settime_ns_is_present (surface)."""
import time

assert hasattr(time, "clock_settime_ns")
print("api_clock_settime_ns_is_present OK")
