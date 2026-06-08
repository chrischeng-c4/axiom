# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_listdir_is_present"
# subject = "os.listdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.listdir: api_listdir_is_present (surface)."""
import os

assert hasattr(os, "listdir")
print("api_listdir_is_present OK")
