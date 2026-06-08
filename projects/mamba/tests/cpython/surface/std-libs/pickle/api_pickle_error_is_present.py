# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pickle_error_is_present"
# subject = "pickle.PickleError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.PickleError: api_pickle_error_is_present (surface)."""
import pickle

assert hasattr(pickle, "PickleError")
print("api_pickle_error_is_present OK")
