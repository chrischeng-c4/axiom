# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_setreuid_is_present"
# subject = "os.setreuid"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.setreuid: api_setreuid_is_present (surface)."""
import os

assert hasattr(os, "setreuid")
print("api_setreuid_is_present OK")
