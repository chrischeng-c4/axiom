# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_p_all_is_present"
# subject = "os.P_ALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.P_ALL: api_p_all_is_present (surface)."""
import os

assert hasattr(os, "P_ALL")
print("api_p_all_is_present OK")
