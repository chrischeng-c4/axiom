# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_tzset_is_present"
# subject = "time.tzset"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.tzset: api_tzset_is_present (surface)."""
import time

assert hasattr(time, "tzset")
print("api_tzset_is_present OK")
