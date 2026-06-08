# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_false_is_present"
# subject = "pickle.FALSE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.FALSE: api_false_is_present (surface)."""
import pickle

assert hasattr(pickle, "FALSE")
print("api_false_is_present OK")
