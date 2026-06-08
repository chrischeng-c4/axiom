# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_true_is_present"
# subject = "pickle.TRUE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.TRUE: api_true_is_present (surface)."""
import pickle

assert hasattr(pickle, "TRUE")
print("api_true_is_present OK")
