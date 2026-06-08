# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setegid_is_present"
# subject = "os.setegid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setegid: api_setegid_is_present (surface)."""
import os

assert hasattr(os, "setegid")
print("api_setegid_is_present OK")
