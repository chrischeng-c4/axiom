# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_py_dll_is_present"
# subject = "ctypes.PyDLL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.PyDLL: api_py_dll_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "PyDLL")
print("api_py_dll_is_present OK")
