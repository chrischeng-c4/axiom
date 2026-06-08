# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_strftime_is_present"
# subject = "time.strftime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.strftime: api_strftime_is_present (surface)."""
import time

assert hasattr(time, "strftime")
print("api_strftime_is_present OK")
