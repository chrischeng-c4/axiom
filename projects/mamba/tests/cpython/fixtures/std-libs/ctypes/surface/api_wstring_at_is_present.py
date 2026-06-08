# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_wstring_at_is_present"
# subject = "ctypes.wstring_at"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.wstring_at: api_wstring_at_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "wstring_at")
print("api_wstring_at_is_present OK")
