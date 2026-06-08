# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_exc_info_is_present"
# subject = "sys.exc_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.exc_info: api_exc_info_is_present (surface)."""
import sys

assert hasattr(sys, "exc_info")
print("api_exc_info_is_present OK")
