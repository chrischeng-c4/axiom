# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_extsep_is_present"
# subject = "os.extsep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.extsep: api_extsep_is_present (surface)."""
import os

assert hasattr(os, "extsep")
print("api_extsep_is_present OK")
