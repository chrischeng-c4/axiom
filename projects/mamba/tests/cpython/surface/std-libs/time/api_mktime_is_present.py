# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_mktime_is_present"
# subject = "time.mktime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.mktime: api_mktime_is_present (surface)."""
import time

assert hasattr(time, "mktime")
print("api_mktime_is_present OK")
