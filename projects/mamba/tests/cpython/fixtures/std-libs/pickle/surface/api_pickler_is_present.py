# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pickler_is_present"
# subject = "pickle.Pickler"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.Pickler: api_pickler_is_present (surface)."""
import pickle

assert hasattr(pickle, "Pickler")
print("api_pickler_is_present OK")
