# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_frozenset_is_present"
# subject = "pickle.FROZENSET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.FROZENSET: api_frozenset_is_present (surface)."""
import pickle

assert hasattr(pickle, "FROZENSET")
print("api_frozenset_is_present OK")
