# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_ctime_is_present"
# subject = "time.ctime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.ctime: api_ctime_is_present (surface)."""
import time

assert hasattr(time, "ctime")
print("api_ctime_is_present OK")
