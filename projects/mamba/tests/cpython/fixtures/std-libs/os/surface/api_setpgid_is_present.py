# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setpgid_is_present"
# subject = "os.setpgid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setpgid: api_setpgid_is_present (surface)."""
import os

assert hasattr(os, "setpgid")
print("api_setpgid_is_present OK")
