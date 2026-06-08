# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_pathsep_is_present"
# subject = "os.pathsep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.pathsep: api_pathsep_is_present (surface)."""
import os

assert hasattr(os, "pathsep")
print("api_pathsep_is_present OK")
