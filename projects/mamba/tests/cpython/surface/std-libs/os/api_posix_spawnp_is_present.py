# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_posix_spawnp_is_present"
# subject = "os.posix_spawnp"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.posix_spawnp: api_posix_spawnp_is_present (surface)."""
import os

assert hasattr(os, "posix_spawnp")
print("api_posix_spawnp_is_present OK")
