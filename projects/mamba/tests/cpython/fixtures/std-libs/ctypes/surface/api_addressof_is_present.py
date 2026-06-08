# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_addressof_is_present"
# subject = "ctypes.addressof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.addressof: api_addressof_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "addressof")
print("api_addressof_is_present OK")
