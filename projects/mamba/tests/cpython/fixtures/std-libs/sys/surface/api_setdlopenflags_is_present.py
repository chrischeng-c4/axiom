# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_setdlopenflags_is_present"
# subject = "sys.setdlopenflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.setdlopenflags: api_setdlopenflags_is_present (surface)."""
import sys

assert hasattr(sys, "setdlopenflags")
print("api_setdlopenflags_is_present OK")
