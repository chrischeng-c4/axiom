# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_string_at_is_present"
# subject = "ctypes.string_at"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.string_at: api_string_at_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "string_at")
print("api_string_at_is_present OK")
