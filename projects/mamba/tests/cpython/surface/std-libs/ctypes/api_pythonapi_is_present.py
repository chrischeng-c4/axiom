# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_pythonapi_is_present"
# subject = "ctypes.pythonapi"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.pythonapi: api_pythonapi_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "pythonapi")
print("api_pythonapi_is_present OK")
