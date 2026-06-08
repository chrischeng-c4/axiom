# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pickling_error_is_present"
# subject = "pickle.PicklingError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.PicklingError: api_pickling_error_is_present (surface)."""
import pickle

assert hasattr(pickle, "PicklingError")
print("api_pickling_error_is_present OK")
