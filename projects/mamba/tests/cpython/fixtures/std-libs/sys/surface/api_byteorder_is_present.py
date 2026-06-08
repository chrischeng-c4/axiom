# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_byteorder_is_present"
# subject = "sys.byteorder"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.byteorder: api_byteorder_is_present (surface)."""
import sys

assert hasattr(sys, "byteorder")
print("api_byteorder_is_present OK")
