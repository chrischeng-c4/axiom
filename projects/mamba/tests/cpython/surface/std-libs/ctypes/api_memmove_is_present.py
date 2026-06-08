# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_memmove_is_present"
# subject = "ctypes.memmove"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.memmove: api_memmove_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "memmove")
print("api_memmove_is_present OK")
