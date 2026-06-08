# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_sizeof_is_present"
# subject = "ctypes.sizeof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.sizeof: api_sizeof_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "sizeof")
print("api_sizeof_is_present OK")
