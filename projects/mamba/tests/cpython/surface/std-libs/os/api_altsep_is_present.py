# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_altsep_is_present"
# subject = "os.altsep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.altsep: api_altsep_is_present (surface)."""
import os

assert hasattr(os, "altsep")
print("api_altsep_is_present OK")
