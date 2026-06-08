# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnve_is_present"
# subject = "os.spawnve"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnve: api_spawnve_is_present (surface)."""
import os

assert hasattr(os, "spawnve")
print("api_spawnve_is_present OK")
