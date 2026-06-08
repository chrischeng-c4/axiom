# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getegid_is_present"
# subject = "os.getegid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getegid: api_getegid_is_present (surface)."""
import os

assert hasattr(os, "getegid")
print("api_getegid_is_present OK")
