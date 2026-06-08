# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ttyname_is_present"
# subject = "os.ttyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.ttyname: api_ttyname_is_present (surface)."""
import os

assert hasattr(os, "ttyname")
print("api_ttyname_is_present OK")
