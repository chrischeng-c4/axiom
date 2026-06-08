# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_wuntraced_is_present"
# subject = "os.WUNTRACED"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.WUNTRACED: api_wuntraced_is_present (surface)."""
import os

assert hasattr(os, "WUNTRACED")
print("api_wuntraced_is_present OK")
