# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getallocatedblocks_is_present"
# subject = "sys.getallocatedblocks"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getallocatedblocks: api_getallocatedblocks_is_present (surface)."""
import sys

assert hasattr(sys, "getallocatedblocks")
print("api_getallocatedblocks_is_present OK")
