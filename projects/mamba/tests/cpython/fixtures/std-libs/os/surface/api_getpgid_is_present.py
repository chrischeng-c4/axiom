# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getpgid_is_present"
# subject = "os.getpgid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getpgid: api_getpgid_is_present (surface)."""
import os

assert hasattr(os, "getpgid")
print("api_getpgid_is_present OK")
