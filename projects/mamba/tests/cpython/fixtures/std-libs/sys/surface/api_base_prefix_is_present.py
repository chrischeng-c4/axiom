# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_base_prefix_is_present"
# subject = "sys.base_prefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.base_prefix: api_base_prefix_is_present (surface)."""
import sys

assert hasattr(sys, "base_prefix")
print("api_base_prefix_is_present OK")
