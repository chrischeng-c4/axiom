# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_c_wchar_is_present"
# subject = "ctypes.c_wchar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.c_wchar: api_c_wchar_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "c_wchar")
print("api_c_wchar_is_present OK")
