# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getpriority_is_present"
# subject = "os.getpriority"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getpriority: api_getpriority_is_present (surface)."""
import os

assert hasattr(os, "getpriority")
print("api_getpriority_is_present OK")
