# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_abiflags_is_present"
# subject = "sys.abiflags"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.abiflags: api_abiflags_is_present (surface)."""
import sys

assert hasattr(sys, "abiflags")
print("api_abiflags_is_present OK")
