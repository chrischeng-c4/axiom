# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fdopen_is_present"
# subject = "os.fdopen"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fdopen: api_fdopen_is_present (surface)."""
import os

assert hasattr(os, "fdopen")
print("api_fdopen_is_present OK")
