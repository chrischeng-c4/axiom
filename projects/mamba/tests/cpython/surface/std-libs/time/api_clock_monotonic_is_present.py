# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_monotonic_is_present"
# subject = "time.CLOCK_MONOTONIC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.CLOCK_MONOTONIC: api_clock_monotonic_is_present (surface)."""
import time

assert hasattr(time, "CLOCK_MONOTONIC")
print("api_clock_monotonic_is_present OK")
