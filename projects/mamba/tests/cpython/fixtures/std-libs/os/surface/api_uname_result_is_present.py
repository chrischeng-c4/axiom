# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_uname_result_is_present"
# subject = "os.uname_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.uname_result: api_uname_result_is_present (surface)."""
import os

assert hasattr(os, "uname_result")
print("api_uname_result_is_present OK")
