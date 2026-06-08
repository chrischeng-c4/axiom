# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_default_mode_is_present"
# subject = "ctypes.DEFAULT_MODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.DEFAULT_MODE: api_default_mode_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "DEFAULT_MODE")
print("api_default_mode_is_present OK")
