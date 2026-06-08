# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setgid_is_present"
# subject = "os.setgid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setgid: api_setgid_is_present (surface)."""
import os

assert hasattr(os, "setgid")
print("api_setgid_is_present OK")
