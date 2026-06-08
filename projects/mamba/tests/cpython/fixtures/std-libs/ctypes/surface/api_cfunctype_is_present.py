# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_cfunctype_is_present"
# subject = "ctypes.CFUNCTYPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.CFUNCTYPE: api_cfunctype_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "CFUNCTYPE")
print("api_cfunctype_is_present OK")
