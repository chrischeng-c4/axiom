# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_mark_is_present"
# subject = "pickle.MARK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.MARK: api_mark_is_present (surface)."""
import pickle

assert hasattr(pickle, "MARK")
print("api_mark_is_present OK")
