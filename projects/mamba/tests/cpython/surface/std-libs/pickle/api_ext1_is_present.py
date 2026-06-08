# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_ext1_is_present"
# subject = "pickle.EXT1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.EXT1: api_ext1_is_present (surface)."""
import pickle

assert hasattr(pickle, "EXT1")
print("api_ext1_is_present OK")
