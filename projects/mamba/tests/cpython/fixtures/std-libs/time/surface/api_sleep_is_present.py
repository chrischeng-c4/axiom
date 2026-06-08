# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_sleep_is_present"
# subject = "time.sleep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.sleep: api_sleep_is_present (surface)."""
import time

assert hasattr(time, "sleep")
print("api_sleep_is_present OK")
