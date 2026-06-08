# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_lstat_is_present"
# subject = "os.lstat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.lstat: api_lstat_is_present (surface)."""
import os

assert hasattr(os, "lstat")
print("api_lstat_is_present OK")
