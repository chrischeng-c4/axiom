# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setuid_is_present"
# subject = "os.setuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setuid: api_setuid_is_present (surface)."""
import os

assert hasattr(os, "setuid")
print("api_setuid_is_present OK")
