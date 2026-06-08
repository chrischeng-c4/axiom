# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_pardir_is_present"
# subject = "os.pardir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.pardir: api_pardir_is_present (surface)."""
import os

assert hasattr(os, "pardir")
print("api_pardir_is_present OK")
