# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_pointer_is_present_2"
# subject = "ctypes.pointer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.pointer: api_pointer_is_present_2 (surface)."""
import ctypes

assert hasattr(ctypes, "pointer")
print("api_pointer_is_present_2 OK")
