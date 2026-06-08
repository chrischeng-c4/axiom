# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_gmtime_is_present"
# subject = "time.gmtime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.gmtime: api_gmtime_is_present (surface)."""
import time

assert hasattr(time, "gmtime")
print("api_gmtime_is_present OK")
