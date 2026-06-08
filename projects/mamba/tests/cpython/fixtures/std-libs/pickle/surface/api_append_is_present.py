# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_append_is_present"
# subject = "pickle.APPEND"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.APPEND: api_append_is_present (surface)."""
import pickle

assert hasattr(pickle, "APPEND")
print("api_append_is_present OK")
