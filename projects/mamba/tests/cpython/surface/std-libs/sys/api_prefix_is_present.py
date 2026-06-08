# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_prefix_is_present"
# subject = "sys.prefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.prefix: api_prefix_is_present (surface)."""
import sys

assert hasattr(sys, "prefix")
print("api_prefix_is_present OK")
