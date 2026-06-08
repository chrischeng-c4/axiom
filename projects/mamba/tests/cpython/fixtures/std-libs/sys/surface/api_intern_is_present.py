# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_intern_is_present"
# subject = "sys.intern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.intern: api_intern_is_present (surface)."""
import sys

assert hasattr(sys, "intern")
print("api_intern_is_present OK")
