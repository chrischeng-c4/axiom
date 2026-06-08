# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_perf_counter_is_present"
# subject = "time.perf_counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.perf_counter: api_perf_counter_is_present (surface)."""
import time

assert hasattr(time, "perf_counter")
print("api_perf_counter_is_present OK")
