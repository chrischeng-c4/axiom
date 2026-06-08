# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_long_is_present"
# subject = "pickle.LONG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.LONG: api_long_is_present (surface)."""
import pickle

assert hasattr(pickle, "LONG")
print("api_long_is_present OK")
