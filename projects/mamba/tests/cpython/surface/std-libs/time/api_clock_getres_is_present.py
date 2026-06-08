# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_clock_getres_is_present"
# subject = "time.clock_getres"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.clock_getres: api_clock_getres_is_present (surface)."""
import time

assert hasattr(time, "clock_getres")
print("api_clock_getres_is_present OK")
