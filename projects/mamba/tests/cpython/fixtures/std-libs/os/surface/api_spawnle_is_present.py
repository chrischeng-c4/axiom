# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnle_is_present"
# subject = "os.spawnle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnle: api_spawnle_is_present (surface)."""
import os

assert hasattr(os, "spawnle")
print("api_spawnle_is_present OK")
