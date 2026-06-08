# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setregid_is_present"
# subject = "os.setregid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setregid: api_setregid_is_present (surface)."""
import os

assert hasattr(os, "setregid")
print("api_setregid_is_present OK")
