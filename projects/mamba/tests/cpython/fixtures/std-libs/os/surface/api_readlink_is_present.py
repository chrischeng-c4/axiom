# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_readlink_is_present"
# subject = "os.readlink"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.readlink: api_readlink_is_present (surface)."""
import os

assert hasattr(os, "readlink")
print("api_readlink_is_present OK")
