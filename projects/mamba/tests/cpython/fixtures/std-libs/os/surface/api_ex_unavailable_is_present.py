# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ex_unavailable_is_present"
# subject = "os.EX_UNAVAILABLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.EX_UNAVAILABLE: api_ex_unavailable_is_present (surface)."""
import os

assert hasattr(os, "EX_UNAVAILABLE")
print("api_ex_unavailable_is_present OK")
