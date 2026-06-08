# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_byref_is_present"
# subject = "ctypes.byref"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.byref: api_byref_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "byref")
print("api_byref_is_present OK")
