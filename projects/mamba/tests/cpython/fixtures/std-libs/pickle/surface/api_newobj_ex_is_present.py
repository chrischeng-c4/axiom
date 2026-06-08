# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_newobj_ex_is_present"
# subject = "pickle.NEWOBJ_EX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.NEWOBJ_EX: api_newobj_ex_is_present (surface)."""
import pickle

assert hasattr(pickle, "NEWOBJ_EX")
print("api_newobj_ex_is_present OK")
