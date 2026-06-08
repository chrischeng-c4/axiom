# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_rtld_global_is_present"
# subject = "ctypes.RTLD_GLOBAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.RTLD_GLOBAL: api_rtld_global_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "RTLD_GLOBAL")
print("api_rtld_global_is_present OK")
