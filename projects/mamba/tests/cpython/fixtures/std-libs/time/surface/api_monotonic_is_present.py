# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_monotonic_is_present"
# subject = "time.monotonic"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.monotonic: api_monotonic_is_present (surface)."""
import time

assert hasattr(time, "monotonic")
print("api_monotonic_is_present OK")
