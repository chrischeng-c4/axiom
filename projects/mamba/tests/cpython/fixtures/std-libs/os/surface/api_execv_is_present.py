# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execv_is_present"
# subject = "os.execv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execv: api_execv_is_present (surface)."""
import os

assert hasattr(os, "execv")
print("api_execv_is_present OK")
