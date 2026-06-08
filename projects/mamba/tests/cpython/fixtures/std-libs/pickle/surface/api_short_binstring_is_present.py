# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_short_binstring_is_present"
# subject = "pickle.SHORT_BINSTRING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.SHORT_BINSTRING: api_short_binstring_is_present (surface)."""
import pickle

assert hasattr(pickle, "SHORT_BINSTRING")
print("api_short_binstring_is_present OK")
