# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_exec_prefix_is_present"
# subject = "sys.exec_prefix"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.exec_prefix: api_exec_prefix_is_present (surface)."""
import sys

assert hasattr(sys, "exec_prefix")
print("api_exec_prefix_is_present OK")
