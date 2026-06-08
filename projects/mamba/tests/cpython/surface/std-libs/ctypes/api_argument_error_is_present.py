# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_argument_error_is_present"
# subject = "ctypes.ArgumentError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.ArgumentError: api_argument_error_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "ArgumentError")
print("api_argument_error_is_present OK")
