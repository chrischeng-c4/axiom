# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_confstr_names_is_present"
# subject = "os.confstr_names"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.confstr_names: api_confstr_names_is_present (surface)."""
import os

assert hasattr(os, "confstr_names")
print("api_confstr_names_is_present OK")
