# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_curdir_is_present"
# subject = "os.curdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.curdir: api_curdir_is_present (surface)."""
import os

assert hasattr(os, "curdir")
print("api_curdir_is_present OK")
