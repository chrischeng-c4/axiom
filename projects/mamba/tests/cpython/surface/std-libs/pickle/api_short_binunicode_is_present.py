# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_short_binunicode_is_present"
# subject = "pickle.SHORT_BINUNICODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.SHORT_BINUNICODE: api_short_binunicode_is_present (surface)."""
import pickle

assert hasattr(pickle, "SHORT_BINUNICODE")
print("api_short_binunicode_is_present OK")
