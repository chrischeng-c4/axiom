# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_little_endian_structure_is_present"
# subject = "ctypes.LittleEndianStructure"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.LittleEndianStructure: api_little_endian_structure_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "LittleEndianStructure")
print("api_little_endian_structure_is_present OK")
