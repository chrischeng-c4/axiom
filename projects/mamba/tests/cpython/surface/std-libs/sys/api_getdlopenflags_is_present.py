# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getdlopenflags_is_present"
# subject = "sys.getdlopenflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getdlopenflags: api_getdlopenflags_is_present (surface)."""
import sys

assert hasattr(sys, "getdlopenflags")
print("api_getdlopenflags_is_present OK")
