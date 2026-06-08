# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getppid_is_present"
# subject = "os.getppid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getppid: api_getppid_is_present (surface)."""
import os

assert hasattr(os, "getppid")
print("api_getppid_is_present OK")
