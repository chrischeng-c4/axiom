# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_process_time_is_present"
# subject = "time.process_time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.process_time: api_process_time_is_present (surface)."""
import time

assert hasattr(time, "process_time")
print("api_process_time_is_present OK")
