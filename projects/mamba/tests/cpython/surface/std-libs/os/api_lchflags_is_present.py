# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_lchflags_is_present"
# subject = "os.lchflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.lchflags: api_lchflags_is_present (surface)."""
import os

assert hasattr(os, "lchflags")
print("api_lchflags_is_present OK")
