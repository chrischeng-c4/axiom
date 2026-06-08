# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_devnull_is_present"
# subject = "os.devnull"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.devnull: api_devnull_is_present (surface)."""
import os

assert hasattr(os, "devnull")
print("api_devnull_is_present OK")
