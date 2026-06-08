# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_mknod_is_present"
# subject = "os.mknod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.mknod: api_mknod_is_present (surface)."""
import os

assert hasattr(os, "mknod")
print("api_mknod_is_present OK")
