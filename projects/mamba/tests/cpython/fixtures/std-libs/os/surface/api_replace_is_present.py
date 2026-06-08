# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_replace_is_present"
# subject = "os.replace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.replace: api_replace_is_present (surface)."""
import os

assert hasattr(os, "replace")
print("api_replace_is_present OK")
