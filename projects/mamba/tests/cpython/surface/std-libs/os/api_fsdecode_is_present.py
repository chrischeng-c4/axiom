# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_fsdecode_is_present"
# subject = "os.fsdecode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.fsdecode: api_fsdecode_is_present (surface)."""
import os

assert hasattr(os, "fsdecode")
print("api_fsdecode_is_present OK")
