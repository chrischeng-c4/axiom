# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_killpg_is_present"
# subject = "os.killpg"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.killpg: api_killpg_is_present (surface)."""
import os

assert hasattr(os, "killpg")
print("api_killpg_is_present OK")
