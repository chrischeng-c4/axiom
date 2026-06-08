# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binstring_is_present"
# subject = "pickle.BINSTRING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINSTRING: api_binstring_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINSTRING")
print("api_binstring_is_present OK")
