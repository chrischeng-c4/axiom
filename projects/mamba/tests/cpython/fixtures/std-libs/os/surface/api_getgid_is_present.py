# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getgid_is_present"
# subject = "os.getgid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getgid: api_getgid_is_present (surface)."""
import os

assert hasattr(os, "getgid")
print("api_getgid_is_present OK")
