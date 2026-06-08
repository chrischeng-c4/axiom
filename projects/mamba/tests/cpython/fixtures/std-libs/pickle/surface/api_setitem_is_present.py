# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_setitem_is_present"
# subject = "pickle.SETITEM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.SETITEM: api_setitem_is_present (surface)."""
import pickle

assert hasattr(pickle, "SETITEM")
print("api_setitem_is_present OK")
