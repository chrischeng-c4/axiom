# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fstat_is_present"
# subject = "os.fstat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fstat: api_fstat_is_present (surface)."""
import os

assert hasattr(os, "fstat")
print("api_fstat_is_present OK")
