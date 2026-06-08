# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getuid_is_present"
# subject = "os.getuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getuid: api_getuid_is_present (surface)."""
import os

assert hasattr(os, "getuid")
print("api_getuid_is_present OK")
