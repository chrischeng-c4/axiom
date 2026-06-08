# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_empty_set_is_present"
# subject = "pickle.EMPTY_SET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.EMPTY_SET: api_empty_set_is_present (surface)."""
import pickle

assert hasattr(pickle, "EMPTY_SET")
print("api_empty_set_is_present OK")
