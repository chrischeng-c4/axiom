# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_stderr_is_present"
# subject = "sys.stderr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.stderr: api_stderr_is_present (surface)."""
import sys

assert hasattr(sys, "stderr")
print("api_stderr_is_present OK")
