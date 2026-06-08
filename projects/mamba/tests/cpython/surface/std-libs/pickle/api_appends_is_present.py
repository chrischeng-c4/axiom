# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_appends_is_present"
# subject = "pickle.APPENDS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.APPENDS: api_appends_is_present (surface)."""
import pickle

assert hasattr(pickle, "APPENDS")
print("api_appends_is_present OK")
