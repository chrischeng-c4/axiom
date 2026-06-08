# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_chflags_is_present"
# subject = "os.chflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.chflags: api_chflags_is_present (surface)."""
import os

assert hasattr(os, "chflags")
print("api_chflags_is_present OK")
