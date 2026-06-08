# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_initgroups_is_present"
# subject = "os.initgroups"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.initgroups: api_initgroups_is_present (surface)."""
import os

assert hasattr(os, "initgroups")
print("api_initgroups_is_present OK")
