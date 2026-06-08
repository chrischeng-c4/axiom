# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_flags_is_present"
# subject = "sys.flags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.flags: api_flags_is_present (surface)."""
import sys

assert hasattr(sys, "flags")
print("api_flags_is_present OK")
