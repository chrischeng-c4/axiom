# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getrecursionlimit_is_present"
# subject = "sys.getrecursionlimit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getrecursionlimit: api_getrecursionlimit_is_present (surface)."""
import sys

assert hasattr(sys, "getrecursionlimit")
print("api_getrecursionlimit_is_present OK")
