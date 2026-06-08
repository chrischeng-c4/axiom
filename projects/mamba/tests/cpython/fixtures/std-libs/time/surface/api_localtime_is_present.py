# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_localtime_is_present"
# subject = "time.localtime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.localtime: api_localtime_is_present (surface)."""
import time

assert hasattr(time, "localtime")
print("api_localtime_is_present OK")
