# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_closerange_is_present"
# subject = "os.closerange"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.closerange: api_closerange_is_present (surface)."""
import os

assert hasattr(os, "closerange")
print("api_closerange_is_present OK")
