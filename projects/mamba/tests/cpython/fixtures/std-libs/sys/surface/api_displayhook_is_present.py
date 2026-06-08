# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_displayhook_is_present"
# subject = "sys.displayhook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.displayhook: api_displayhook_is_present (surface)."""
import sys

assert hasattr(sys, "displayhook")
print("api_displayhook_is_present OK")
