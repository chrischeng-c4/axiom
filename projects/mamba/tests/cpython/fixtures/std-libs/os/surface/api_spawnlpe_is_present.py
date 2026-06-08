# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnlpe_is_present"
# subject = "os.spawnlpe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnlpe: api_spawnlpe_is_present (surface)."""
import os

assert hasattr(os, "spawnlpe")
print("api_spawnlpe_is_present OK")
