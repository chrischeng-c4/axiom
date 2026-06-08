# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_symlink_is_present"
# subject = "os.symlink"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.symlink: api_symlink_is_present (surface)."""
import os

assert hasattr(os, "symlink")
print("api_symlink_is_present OK")
