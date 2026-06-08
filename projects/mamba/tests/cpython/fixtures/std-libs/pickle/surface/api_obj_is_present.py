# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_obj_is_present"
# subject = "pickle.OBJ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.OBJ: api_obj_is_present (surface)."""
import pickle

assert hasattr(pickle, "OBJ")
print("api_obj_is_present OK")
