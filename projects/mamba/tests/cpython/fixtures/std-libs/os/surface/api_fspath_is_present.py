# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fspath_is_present"
# subject = "os.fspath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fspath: api_fspath_is_present (surface)."""
import os

assert hasattr(os, "fspath")
print("api_fspath_is_present OK")
