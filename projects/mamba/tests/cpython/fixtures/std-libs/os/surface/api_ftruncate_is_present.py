# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ftruncate_is_present"
# subject = "os.ftruncate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.ftruncate: api_ftruncate_is_present (surface)."""
import os

assert hasattr(os, "ftruncate")
print("api_ftruncate_is_present OK")
