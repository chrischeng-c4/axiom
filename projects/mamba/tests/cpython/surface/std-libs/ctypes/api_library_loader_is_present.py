# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_library_loader_is_present"
# subject = "ctypes.LibraryLoader"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.LibraryLoader: api_library_loader_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "LibraryLoader")
print("api_library_loader_is_present OK")
