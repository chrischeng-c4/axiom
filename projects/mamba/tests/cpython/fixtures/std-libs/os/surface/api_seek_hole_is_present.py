# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_seek_hole_is_present"
# subject = "os.SEEK_HOLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.SEEK_HOLE: api_seek_hole_is_present (surface)."""
import os

assert hasattr(os, "SEEK_HOLE")
print("api_seek_hole_is_present OK")
