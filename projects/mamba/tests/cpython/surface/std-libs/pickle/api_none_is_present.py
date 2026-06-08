# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_none_is_present"
# subject = "pickle.NONE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.NONE: api_none_is_present (surface)."""
import pickle

assert hasattr(pickle, "NONE")
print("api_none_is_present OK")
