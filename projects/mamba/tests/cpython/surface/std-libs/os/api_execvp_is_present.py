# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execvp_is_present"
# subject = "os.execvp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execvp: api_execvp_is_present (surface)."""
import os

assert hasattr(os, "execvp")
print("api_execvp_is_present OK")
