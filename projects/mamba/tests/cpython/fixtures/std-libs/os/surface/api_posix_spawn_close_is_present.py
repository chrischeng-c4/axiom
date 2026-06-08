# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_posix_spawn_close_is_present"
# subject = "os.POSIX_SPAWN_CLOSE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.POSIX_SPAWN_CLOSE: api_posix_spawn_close_is_present (surface)."""
import os

assert hasattr(os, "POSIX_SPAWN_CLOSE")
print("api_posix_spawn_close_is_present OK")
