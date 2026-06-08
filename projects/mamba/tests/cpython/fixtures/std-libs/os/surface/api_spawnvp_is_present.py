# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_spawnvp_is_present"
# subject = "os.spawnvp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.spawnvp: api_spawnvp_is_present (surface)."""
import os

assert hasattr(os, "spawnvp")
print("api_spawnvp_is_present OK")
