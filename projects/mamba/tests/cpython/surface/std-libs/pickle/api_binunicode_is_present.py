# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binunicode_is_present"
# subject = "pickle.BINUNICODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINUNICODE: api_binunicode_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINUNICODE")
print("api_binunicode_is_present OK")
