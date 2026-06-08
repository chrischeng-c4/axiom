# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_exit_is_present"
# subject = "sys.exit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.exit: api_exit_is_present (surface)."""
import sys

assert hasattr(sys, "exit")
print("api_exit_is_present OK")
