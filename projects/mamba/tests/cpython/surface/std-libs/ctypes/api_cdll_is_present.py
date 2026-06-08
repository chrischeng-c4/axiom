# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_cdll_is_present"
# subject = "ctypes.CDLL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.CDLL: api_cdll_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "CDLL")
print("api_cdll_is_present OK")
