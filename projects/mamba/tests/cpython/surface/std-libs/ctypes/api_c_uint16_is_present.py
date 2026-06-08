# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_c_uint16_is_present"
# subject = "ctypes.c_uint16"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.c_uint16: api_c_uint16_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "c_uint16")
print("api_c_uint16_is_present OK")
