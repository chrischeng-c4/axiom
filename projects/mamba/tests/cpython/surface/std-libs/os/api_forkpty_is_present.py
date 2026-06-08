# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_forkpty_is_present"
# subject = "os.forkpty"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.forkpty: api_forkpty_is_present (surface)."""
import os

assert hasattr(os, "forkpty")
print("api_forkpty_is_present OK")
