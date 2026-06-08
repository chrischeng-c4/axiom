# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_memset_is_present"
# subject = "ctypes.memset"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.memset: api_memset_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "memset")
print("api_memset_is_present OK")
