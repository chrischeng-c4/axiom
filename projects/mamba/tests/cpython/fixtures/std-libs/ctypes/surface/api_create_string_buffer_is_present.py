# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_create_string_buffer_is_present"
# subject = "ctypes.create_string_buffer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.create_string_buffer: api_create_string_buffer_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "create_string_buffer")
print("api_create_string_buffer_is_present OK")
