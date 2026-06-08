# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_pwrite_is_present"
# subject = "os.pwrite"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.pwrite: api_pwrite_is_present (surface)."""
import os

assert hasattr(os, "pwrite")
print("api_pwrite_is_present OK")
