# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_py_object_is_present"
# subject = "ctypes.py_object"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.py_object: api_py_object_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "py_object")
print("api_py_object_is_present OK")
