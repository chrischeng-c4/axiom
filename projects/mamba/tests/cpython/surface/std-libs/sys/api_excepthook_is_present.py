# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_excepthook_is_present"
# subject = "sys.excepthook"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.excepthook: api_excepthook_is_present (surface)."""
import sys

assert hasattr(sys, "excepthook")
print("api_excepthook_is_present OK")
