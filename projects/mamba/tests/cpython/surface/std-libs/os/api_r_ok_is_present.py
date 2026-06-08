# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_r_ok_is_present"
# subject = "os.R_OK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.R_OK: api_r_ok_is_present (surface)."""
import os

assert hasattr(os, "R_OK")
print("api_r_ok_is_present OK")
