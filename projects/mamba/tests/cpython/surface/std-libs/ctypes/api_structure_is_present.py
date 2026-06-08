# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_structure_is_present"
# subject = "ctypes.Structure"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.Structure: api_structure_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "Structure")
print("api_structure_is_present OK")
