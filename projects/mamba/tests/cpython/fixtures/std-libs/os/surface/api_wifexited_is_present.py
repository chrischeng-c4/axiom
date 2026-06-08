# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wifexited_is_present"
# subject = "os.WIFEXITED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WIFEXITED: api_wifexited_is_present (surface)."""
import os

assert hasattr(os, "WIFEXITED")
print("api_wifexited_is_present OK")
