# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_unlink_is_present"
# subject = "os.unlink"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.unlink: api_unlink_is_present (surface)."""
import os

assert hasattr(os, "unlink")
print("api_unlink_is_present OK")
