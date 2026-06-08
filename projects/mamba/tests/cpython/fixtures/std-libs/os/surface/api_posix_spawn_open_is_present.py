# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_posix_spawn_open_is_present"
# subject = "os.POSIX_SPAWN_OPEN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.POSIX_SPAWN_OPEN: api_posix_spawn_open_is_present (surface)."""
import os

assert hasattr(os, "POSIX_SPAWN_OPEN")
print("api_posix_spawn_open_is_present OK")
