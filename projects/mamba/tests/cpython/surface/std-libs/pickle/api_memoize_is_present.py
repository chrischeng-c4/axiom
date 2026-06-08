# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_memoize_is_present"
# subject = "pickle.MEMOIZE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.MEMOIZE: api_memoize_is_present (surface)."""
import pickle

assert hasattr(pickle, "MEMOIZE")
print("api_memoize_is_present OK")
