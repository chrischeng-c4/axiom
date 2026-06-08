# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_process_time_ns_is_present"
# subject = "time.process_time_ns"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.process_time_ns: api_process_time_ns_is_present (surface)."""
import time

assert hasattr(time, "process_time_ns")
print("api_process_time_ns_is_present OK")
