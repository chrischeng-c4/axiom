# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binunicode8_is_present"
# subject = "pickle.BINUNICODE8"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINUNICODE8: api_binunicode8_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINUNICODE8")
print("api_binunicode8_is_present OK")
