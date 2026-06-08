# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wstopsig_is_present"
# subject = "os.WSTOPSIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WSTOPSIG: api_wstopsig_is_present (surface)."""
import os

assert hasattr(os, "WSTOPSIG")
print("api_wstopsig_is_present OK")
