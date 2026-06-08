# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_setitems_is_present"
# subject = "pickle.SETITEMS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.SETITEMS: api_setitems_is_present (surface)."""
import pickle

assert hasattr(pickle, "SETITEMS")
print("api_setitems_is_present OK")
