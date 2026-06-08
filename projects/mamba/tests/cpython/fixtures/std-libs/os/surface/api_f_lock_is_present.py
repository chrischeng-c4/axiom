# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_f_lock_is_present"
# subject = "os.F_LOCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.F_LOCK: api_f_lock_is_present (surface)."""
import os

assert hasattr(os, "F_LOCK")
print("api_f_lock_is_present OK")
