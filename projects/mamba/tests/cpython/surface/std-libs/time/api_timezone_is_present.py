# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_timezone_is_present"
# subject = "time.timezone"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.timezone: api_timezone_is_present (surface)."""
import time

assert hasattr(time, "timezone")
print("api_timezone_is_present OK")
