# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getenvb_is_present"
# subject = "os.getenvb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getenvb: api_getenvb_is_present (surface)."""
import os

assert hasattr(os, "getenvb")
print("api_getenvb_is_present OK")
