# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pickle_buffer_is_present"
# subject = "pickle.PickleBuffer"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.PickleBuffer: api_pickle_buffer_is_present (surface)."""
import pickle

assert hasattr(pickle, "PickleBuffer")
print("api_pickle_buffer_is_present OK")
