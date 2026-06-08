# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fwalk_is_present"
# subject = "os.fwalk"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fwalk: api_fwalk_is_present (surface)."""
import os

assert hasattr(os, "fwalk")
print("api_fwalk_is_present OK")
