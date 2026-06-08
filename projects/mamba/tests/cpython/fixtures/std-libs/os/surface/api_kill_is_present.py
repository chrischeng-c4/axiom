# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_kill_is_present"
# subject = "os.kill"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.kill: api_kill_is_present (surface)."""
import os

assert hasattr(os, "kill")
print("api_kill_is_present OK")
