# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_global_is_present"
# subject = "pickle.GLOBAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.GLOBAL: api_global_is_present (surface)."""
import pickle

assert hasattr(pickle, "GLOBAL")
print("api_global_is_present OK")
