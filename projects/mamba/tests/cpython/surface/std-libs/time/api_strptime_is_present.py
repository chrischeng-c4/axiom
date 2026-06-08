# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_strptime_is_present"
# subject = "time.strptime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.strptime: api_strptime_is_present (surface)."""
import time

assert hasattr(time, "strptime")
print("api_strptime_is_present OK")
