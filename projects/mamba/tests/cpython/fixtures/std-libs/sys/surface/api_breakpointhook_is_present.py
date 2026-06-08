# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_breakpointhook_is_present"
# subject = "sys.breakpointhook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.breakpointhook: api_breakpointhook_is_present (surface)."""
import sys

assert hasattr(sys, "breakpointhook")
print("api_breakpointhook_is_present OK")
