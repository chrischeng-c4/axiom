# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_put_is_present"
# subject = "pickle.PUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.PUT: api_put_is_present (surface)."""
import pickle

assert hasattr(pickle, "PUT")
print("api_put_is_present OK")
