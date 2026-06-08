# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_list_is_present"
# subject = "pickle.LIST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.LIST: api_list_is_present (surface)."""
import pickle

assert hasattr(pickle, "LIST")
print("api_list_is_present OK")
