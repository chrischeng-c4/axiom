# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_sendfile_is_present"
# subject = "os.sendfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.sendfile: api_sendfile_is_present (surface)."""
import os

assert hasattr(os, "sendfile")
print("api_sendfile_is_present OK")
