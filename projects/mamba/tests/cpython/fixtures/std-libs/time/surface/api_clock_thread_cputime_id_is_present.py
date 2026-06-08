# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_thread_cputime_id_is_present"
# subject = "time.CLOCK_THREAD_CPUTIME_ID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.CLOCK_THREAD_CPUTIME_ID: api_clock_thread_cputime_id_is_present (surface)."""
import time

assert hasattr(time, "CLOCK_THREAD_CPUTIME_ID")
print("api_clock_thread_cputime_id_is_present OK")
