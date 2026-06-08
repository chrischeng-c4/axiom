# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_tuple3_is_present"
# subject = "pickle.TUPLE3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.TUPLE3: api_tuple3_is_present (surface)."""
import pickle

assert hasattr(pickle, "TUPLE3")
print("api_tuple3_is_present OK")
