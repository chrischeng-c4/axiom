# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_float_is_present"
# subject = "pickle.FLOAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.FLOAT: api_float_is_present (surface)."""
import pickle

assert hasattr(pickle, "FLOAT")
print("api_float_is_present OK")
