# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_altzone_is_present"
# subject = "time.altzone"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.altzone: api_altzone_is_present (surface)."""
import time

assert hasattr(time, "altzone")
print("api_altzone_is_present OK")
