# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_implementation_is_present"
# subject = "sys.implementation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.implementation: api_implementation_is_present (surface)."""
import sys

assert hasattr(sys, "implementation")
print("api_implementation_is_present OK")
