# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_scandir_is_present"
# subject = "os.scandir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.scandir: api_scandir_is_present (surface)."""
import os

assert hasattr(os, "scandir")
print("api_scandir_is_present OK")
