# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fchown_is_present"
# subject = "os.fchown"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fchown: api_fchown_is_present (surface)."""
import os

assert hasattr(os, "fchown")
print("api_fchown_is_present OK")
