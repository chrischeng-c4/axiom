# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getunicodeinternedsize_is_present"
# subject = "sys.getunicodeinternedsize"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getunicodeinternedsize: api_getunicodeinternedsize_is_present (surface)."""
import sys

assert hasattr(sys, "getunicodeinternedsize")
print("api_getunicodeinternedsize_is_present OK")
