# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_dict_is_present"
# subject = "pickle.DICT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.DICT: api_dict_is_present (surface)."""
import pickle

assert hasattr(pickle, "DICT")
print("api_dict_is_present OK")
