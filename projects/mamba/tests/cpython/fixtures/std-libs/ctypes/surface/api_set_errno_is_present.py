# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_set_errno_is_present"
# subject = "ctypes.set_errno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.set_errno: api_set_errno_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "set_errno")
print("api_set_errno_is_present OK")
