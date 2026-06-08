# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_umask_is_present"
# subject = "os.umask"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.umask: api_umask_is_present (surface)."""
import os

assert hasattr(os, "umask")
print("api_umask_is_present OK")
