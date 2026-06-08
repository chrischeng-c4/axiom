# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_time_is_present"
# subject = "time.time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.time: api_time_is_present (surface)."""
import time

assert hasattr(time, "time")
print("api_time_is_present OK")
