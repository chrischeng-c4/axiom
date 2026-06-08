# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_additems_is_present"
# subject = "pickle.ADDITEMS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.ADDITEMS: api_additems_is_present (surface)."""
import pickle

assert hasattr(pickle, "ADDITEMS")
print("api_additems_is_present OK")
