# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_pyfunctype_is_present"
# subject = "ctypes.PYFUNCTYPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.PYFUNCTYPE: api_pyfunctype_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "PYFUNCTYPE")
print("api_pyfunctype_is_present OK")
