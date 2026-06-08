# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_reduce_is_present"
# subject = "pickle.REDUCE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.REDUCE: api_reduce_is_present (surface)."""
import pickle

assert hasattr(pickle, "REDUCE")
print("api_reduce_is_present OK")
