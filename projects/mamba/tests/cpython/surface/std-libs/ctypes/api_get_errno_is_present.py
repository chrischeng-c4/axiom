# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_get_errno_is_present"
# subject = "ctypes.get_errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.get_errno: api_get_errno_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "get_errno")
print("api_get_errno_is_present OK")
