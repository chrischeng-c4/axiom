# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_get_is_present"
# subject = "pickle.GET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.GET: api_get_is_present (surface)."""
import pickle

assert hasattr(pickle, "GET")
print("api_get_is_present OK")
