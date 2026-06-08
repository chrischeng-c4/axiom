# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getloadavg_is_present"
# subject = "os.getloadavg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getloadavg: api_getloadavg_is_present (surface)."""
import os

assert hasattr(os, "getloadavg")
print("api_getloadavg_is_present OK")
