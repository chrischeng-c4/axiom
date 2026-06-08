# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_setprofile_is_present"
# subject = "sys.setprofile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.setprofile: api_setprofile_is_present (surface)."""
import sys

assert hasattr(sys, "setprofile")
print("api_setprofile_is_present OK")
