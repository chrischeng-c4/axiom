# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_nice_is_present"
# subject = "os.nice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.nice: api_nice_is_present (surface)."""
import os

assert hasattr(os, "nice")
print("api_nice_is_present OK")
