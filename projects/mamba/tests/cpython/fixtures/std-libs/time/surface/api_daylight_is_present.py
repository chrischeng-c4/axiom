# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_daylight_is_present"
# subject = "time.daylight"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.daylight: api_daylight_is_present (surface)."""
import time

assert hasattr(time, "daylight")
print("api_daylight_is_present OK")
