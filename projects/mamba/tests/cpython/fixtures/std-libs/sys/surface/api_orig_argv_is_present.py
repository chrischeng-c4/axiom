# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_orig_argv_is_present"
# subject = "sys.orig_argv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.orig_argv: api_orig_argv_is_present (surface)."""
import sys

assert hasattr(sys, "orig_argv")
print("api_orig_argv_is_present OK")
