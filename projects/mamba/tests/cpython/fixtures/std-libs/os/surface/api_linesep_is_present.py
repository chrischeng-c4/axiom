# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_linesep_is_present"
# subject = "os.linesep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.linesep: api_linesep_is_present (surface)."""
import os

assert hasattr(os, "linesep")
print("api_linesep_is_present OK")
