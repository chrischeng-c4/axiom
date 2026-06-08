# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_executable_is_present"
# subject = "sys.executable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.executable: api_executable_is_present (surface)."""
import sys

assert hasattr(sys, "executable")
print("api_executable_is_present OK")
