# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getprofile_is_present"
# subject = "sys.getprofile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getprofile: api_getprofile_is_present (surface)."""
import sys

assert hasattr(sys, "getprofile")
print("api_getprofile_is_present OK")
