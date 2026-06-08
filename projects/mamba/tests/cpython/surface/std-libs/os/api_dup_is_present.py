# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_dup_is_present"
# subject = "os.dup"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.dup: api_dup_is_present (surface)."""
import os

assert hasattr(os, "dup")
print("api_dup_is_present OK")
