# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_truncate_is_present"
# subject = "os.truncate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.truncate: api_truncate_is_present (surface)."""
import os

assert hasattr(os, "truncate")
print("api_truncate_is_present OK")
