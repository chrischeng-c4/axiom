# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_getgroups_is_present"
# subject = "os.getgroups"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.getgroups: api_getgroups_is_present (surface)."""
import os

assert hasattr(os, "getgroups")
print("api_getgroups_is_present OK")
