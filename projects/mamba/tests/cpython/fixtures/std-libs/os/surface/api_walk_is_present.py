# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_walk_is_present"
# subject = "os.walk"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.walk: api_walk_is_present (surface)."""
import os

assert hasattr(os, "walk")
print("api_walk_is_present OK")
