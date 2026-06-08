# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_unpickling_error_is_present"
# subject = "pickle.UnpicklingError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.UnpicklingError: api_unpickling_error_is_present (surface)."""
import pickle

assert hasattr(pickle, "UnpicklingError")
print("api_unpickling_error_is_present OK")
