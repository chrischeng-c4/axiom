# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_int_info_is_present"
# subject = "sys.int_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.int_info: api_int_info_is_present (surface)."""
import sys

assert hasattr(sys, "int_info")
print("api_int_info_is_present OK")
