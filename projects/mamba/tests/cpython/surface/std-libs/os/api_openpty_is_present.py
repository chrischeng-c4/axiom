# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_openpty_is_present"
# subject = "os.openpty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.openpty: api_openpty_is_present (surface)."""
import os

assert hasattr(os, "openpty")
print("api_openpty_is_present OK")
