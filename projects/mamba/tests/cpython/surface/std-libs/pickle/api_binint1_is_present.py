# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binint1_is_present"
# subject = "pickle.BININT1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BININT1: api_binint1_is_present (surface)."""
import pickle

assert hasattr(pickle, "BININT1")
print("api_binint1_is_present OK")
