# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_load_is_present"
# subject = "pickle.load"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.load: api_load_is_present (surface)."""
import pickle

assert hasattr(pickle, "load")
print("api_load_is_present OK")
