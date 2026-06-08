# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_asctime_is_present"
# subject = "time.asctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.asctime: api_asctime_is_present (surface)."""
import time

assert hasattr(time, "asctime")
print("api_asctime_is_present OK")
