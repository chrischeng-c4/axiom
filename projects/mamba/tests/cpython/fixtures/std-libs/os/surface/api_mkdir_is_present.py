# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_mkdir_is_present"
# subject = "os.mkdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.mkdir: api_mkdir_is_present (surface)."""
import os

assert hasattr(os, "mkdir")
print("api_mkdir_is_present OK")
