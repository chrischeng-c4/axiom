# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_bytearray8_is_present"
# subject = "pickle.BYTEARRAY8"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BYTEARRAY8: api_bytearray8_is_present (surface)."""
import pickle

assert hasattr(pickle, "BYTEARRAY8")
print("api_bytearray8_is_present OK")
