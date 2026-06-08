# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_w_ok_is_present"
# subject = "os.W_OK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.W_OK: api_w_ok_is_present (surface)."""
import os

assert hasattr(os, "W_OK")
print("api_w_ok_is_present OK")
