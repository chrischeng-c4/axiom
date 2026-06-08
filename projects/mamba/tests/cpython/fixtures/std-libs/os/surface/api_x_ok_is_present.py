# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_x_ok_is_present"
# subject = "os.X_OK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.X_OK: api_x_ok_is_present (surface)."""
import os

assert hasattr(os, "X_OK")
print("api_x_ok_is_present OK")
