# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setpriority_is_present"
# subject = "os.setpriority"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setpriority: api_setpriority_is_present (surface)."""
import os

assert hasattr(os, "setpriority")
print("api_setpriority_is_present OK")
