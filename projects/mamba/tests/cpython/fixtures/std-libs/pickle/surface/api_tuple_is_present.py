# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_tuple_is_present"
# subject = "pickle.TUPLE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.TUPLE: api_tuple_is_present (surface)."""
import pickle

assert hasattr(pickle, "TUPLE")
print("api_tuple_is_present OK")
