# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_pathconf_is_present"
# subject = "os.pathconf"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.pathconf: api_pathconf_is_present (surface)."""
import os

assert hasattr(os, "pathconf")
print("api_pathconf_is_present OK")
