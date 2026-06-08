# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_maxsize_is_present"
# subject = "sys.maxsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.maxsize: api_maxsize_is_present (surface)."""
import sys

assert hasattr(sys, "maxsize")
print("api_maxsize_is_present OK")
