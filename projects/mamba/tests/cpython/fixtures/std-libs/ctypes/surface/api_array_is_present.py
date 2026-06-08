# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_array_is_present"
# subject = "ctypes.ARRAY"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.ARRAY: api_array_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "ARRAY")
print("api_array_is_present OK")
