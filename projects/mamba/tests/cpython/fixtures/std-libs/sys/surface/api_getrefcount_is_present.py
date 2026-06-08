# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getrefcount_is_present"
# subject = "sys.getrefcount"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getrefcount: api_getrefcount_is_present (surface)."""
import sys

assert hasattr(sys, "getrefcount")
print("api_getrefcount_is_present OK")
