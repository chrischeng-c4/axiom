# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_getsizeof_is_present"
# subject = "sys.getsizeof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.getsizeof: api_getsizeof_is_present (surface)."""
import sys

assert hasattr(sys, "getsizeof")
print("api_getsizeof_is_present OK")
