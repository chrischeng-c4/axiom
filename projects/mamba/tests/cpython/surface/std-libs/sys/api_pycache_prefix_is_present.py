# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_pycache_prefix_is_present"
# subject = "sys.pycache_prefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.pycache_prefix: api_pycache_prefix_is_present (surface)."""
import sys

assert hasattr(sys, "pycache_prefix")
print("api_pycache_prefix_is_present OK")
