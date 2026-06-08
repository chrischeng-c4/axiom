# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_popen_is_present"
# subject = "os.popen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.popen: api_popen_is_present (surface)."""
import os

assert hasattr(os, "popen")
print("api_popen_is_present OK")
