# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_string_is_present"
# subject = "pickle.STRING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.STRING: api_string_is_present (surface)."""
import pickle

assert hasattr(pickle, "STRING")
print("api_string_is_present OK")
