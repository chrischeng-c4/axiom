# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_utime_is_present"
# subject = "os.utime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.utime: api_utime_is_present (surface)."""
import os

assert hasattr(os, "utime")
print("api_utime_is_present OK")
