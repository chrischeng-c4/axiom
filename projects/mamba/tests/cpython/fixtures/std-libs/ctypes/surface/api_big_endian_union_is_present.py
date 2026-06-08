# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_big_endian_union_is_present"
# subject = "ctypes.BigEndianUnion"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.BigEndianUnion: api_big_endian_union_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "BigEndianUnion")
print("api_big_endian_union_is_present OK")
