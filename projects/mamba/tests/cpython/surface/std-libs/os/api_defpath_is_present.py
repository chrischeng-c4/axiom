# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_defpath_is_present"
# subject = "os.defpath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.defpath: api_defpath_is_present (surface)."""
import os

assert hasattr(os, "defpath")
print("api_defpath_is_present OK")
