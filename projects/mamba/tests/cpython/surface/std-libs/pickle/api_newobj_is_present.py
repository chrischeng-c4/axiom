# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_newobj_is_present"
# subject = "pickle.NEWOBJ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.NEWOBJ: api_newobj_is_present (surface)."""
import pickle

assert hasattr(pickle, "NEWOBJ")
print("api_newobj_is_present OK")
