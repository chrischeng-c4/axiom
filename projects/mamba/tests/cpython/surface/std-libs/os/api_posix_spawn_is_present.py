# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_posix_spawn_is_present"
# subject = "os.posix_spawn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.posix_spawn: api_posix_spawn_is_present (surface)."""
import os

assert hasattr(os, "posix_spawn")
print("api_posix_spawn_is_present OK")
