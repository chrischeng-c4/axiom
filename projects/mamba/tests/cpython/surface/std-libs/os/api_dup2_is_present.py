# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_dup2_is_present"
# subject = "os.dup2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.dup2: api_dup2_is_present (surface)."""
import os

assert hasattr(os, "dup2")
print("api_dup2_is_present OK")
