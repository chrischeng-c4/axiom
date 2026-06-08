# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_thread_time_is_present"
# subject = "time.thread_time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.thread_time: api_thread_time_is_present (surface)."""
import time

assert hasattr(time, "thread_time")
print("api_thread_time_is_present OK")
