# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_confstr_is_present"
# subject = "os.confstr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.confstr: api_confstr_is_present (surface)."""
import os

assert hasattr(os, "confstr")
print("api_confstr_is_present OK")
