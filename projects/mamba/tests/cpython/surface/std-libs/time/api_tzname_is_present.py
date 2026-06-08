# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_tzname_is_present"
# subject = "time.tzname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.tzname: api_tzname_is_present (surface)."""
import time

assert hasattr(time, "tzname")
print("api_tzname_is_present OK")
