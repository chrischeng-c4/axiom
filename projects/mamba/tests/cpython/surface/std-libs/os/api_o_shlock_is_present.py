# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_o_shlock_is_present"
# subject = "os.O_SHLOCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.O_SHLOCK: api_o_shlock_is_present (surface)."""
import os

assert hasattr(os, "O_SHLOCK")
print("api_o_shlock_is_present OK")
