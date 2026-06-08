# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_union_is_present"
# subject = "ctypes.Union"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.Union: api_union_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "Union")
print("api_union_is_present OK")
