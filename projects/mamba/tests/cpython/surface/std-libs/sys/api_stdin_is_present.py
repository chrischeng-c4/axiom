# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_stdin_is_present"
# subject = "sys.stdin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.stdin: api_stdin_is_present (surface)."""
import sys

assert hasattr(sys, "stdin")
print("api_stdin_is_present OK")
