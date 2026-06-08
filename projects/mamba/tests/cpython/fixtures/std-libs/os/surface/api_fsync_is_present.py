# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fsync_is_present"
# subject = "os.fsync"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fsync: api_fsync_is_present (surface)."""
import os

assert hasattr(os, "fsync")
print("api_fsync_is_present OK")
