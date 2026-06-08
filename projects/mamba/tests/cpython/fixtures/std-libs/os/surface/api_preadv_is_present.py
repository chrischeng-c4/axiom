# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_preadv_is_present"
# subject = "os.preadv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.preadv: api_preadv_is_present (surface)."""
import os

assert hasattr(os, "preadv")
print("api_preadv_is_present OK")
