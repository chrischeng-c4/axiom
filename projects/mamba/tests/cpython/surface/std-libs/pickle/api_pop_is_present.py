# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pop_is_present"
# subject = "pickle.POP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.POP: api_pop_is_present (surface)."""
import pickle

assert hasattr(pickle, "POP")
print("api_pop_is_present OK")
