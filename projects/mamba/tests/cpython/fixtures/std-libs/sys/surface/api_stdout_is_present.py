# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_stdout_is_present"
# subject = "sys.stdout"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.stdout: api_stdout_is_present (surface)."""
import sys

assert hasattr(sys, "stdout")
print("api_stdout_is_present OK")
