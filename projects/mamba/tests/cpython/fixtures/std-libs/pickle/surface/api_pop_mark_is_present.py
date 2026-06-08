# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_pop_mark_is_present"
# subject = "pickle.POP_MARK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.POP_MARK: api_pop_mark_is_present (surface)."""
import pickle

assert hasattr(pickle, "POP_MARK")
print("api_pop_mark_is_present OK")
