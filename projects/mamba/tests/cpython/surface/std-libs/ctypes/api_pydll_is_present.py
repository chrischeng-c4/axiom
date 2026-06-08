# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_pydll_is_present"
# subject = "ctypes.pydll"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.pydll: api_pydll_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "pydll")
print("api_pydll_is_present OK")
