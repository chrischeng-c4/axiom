# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_hexversion_is_present"
# subject = "sys.hexversion"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.hexversion: api_hexversion_is_present (surface)."""
import sys

assert hasattr(sys, "hexversion")
print("api_hexversion_is_present OK")
