# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_addaudithook_is_present"
# subject = "sys.addaudithook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.addaudithook: api_addaudithook_is_present (surface)."""
import sys

assert hasattr(sys, "addaudithook")
print("api_addaudithook_is_present OK")
