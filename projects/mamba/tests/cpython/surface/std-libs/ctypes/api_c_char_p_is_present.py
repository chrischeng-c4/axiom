# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_c_char_p_is_present"
# subject = "ctypes.c_char_p"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.c_char_p: api_c_char_p_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "c_char_p")
print("api_c_char_p_is_present OK")
