# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_rename_is_present"
# subject = "os.rename"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.rename: api_rename_is_present (surface)."""
import os

assert hasattr(os, "rename")
print("api_rename_is_present OK")
