# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fsencode_is_present"
# subject = "os.fsencode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fsencode: api_fsencode_is_present (surface)."""
import os

assert hasattr(os, "fsencode")
print("api_fsencode_is_present OK")
