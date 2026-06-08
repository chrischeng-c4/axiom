# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_minor_is_present"
# subject = "os.minor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.minor: api_minor_is_present (surface)."""
import os

assert hasattr(os, "minor")
print("api_minor_is_present OK")
