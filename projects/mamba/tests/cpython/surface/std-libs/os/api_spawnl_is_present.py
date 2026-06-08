# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnl_is_present"
# subject = "os.spawnl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnl: api_spawnl_is_present (surface)."""
import os

assert hasattr(os, "spawnl")
print("api_spawnl_is_present OK")
