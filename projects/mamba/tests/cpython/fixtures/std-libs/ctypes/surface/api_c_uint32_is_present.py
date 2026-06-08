# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_c_uint32_is_present"
# subject = "ctypes.c_uint32"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.c_uint32: api_c_uint32_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "c_uint32")
print("api_c_uint32_is_present OK")
