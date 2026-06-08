# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_float_info_is_present"
# subject = "sys.float_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.float_info: api_float_info_is_present (surface)."""
import sys

assert hasattr(sys, "float_info")
print("api_float_info_is_present OK")
