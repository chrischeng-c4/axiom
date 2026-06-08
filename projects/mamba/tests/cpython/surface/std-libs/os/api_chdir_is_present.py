# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_chdir_is_present"
# subject = "os.chdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.chdir: api_chdir_is_present (surface)."""
import os

assert hasattr(os, "chdir")
print("api_chdir_is_present OK")
