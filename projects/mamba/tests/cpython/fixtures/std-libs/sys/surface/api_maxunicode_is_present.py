# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_maxunicode_is_present"
# subject = "sys.maxunicode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.maxunicode: api_maxunicode_is_present (surface)."""
import sys

assert hasattr(sys, "maxunicode")
print("api_maxunicode_is_present OK")
