# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_execve_is_present"
# subject = "os.execve"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.execve: api_execve_is_present (surface)."""
import os

assert hasattr(os, "execve")
print("api_execve_is_present OK")
