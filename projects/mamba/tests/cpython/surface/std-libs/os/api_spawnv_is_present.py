# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnv_is_present"
# subject = "os.spawnv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnv: api_spawnv_is_present (surface)."""
import os

assert hasattr(os, "spawnv")
print("api_spawnv_is_present OK")
