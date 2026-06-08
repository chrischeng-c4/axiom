# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binint_is_present"
# subject = "pickle.BININT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BININT: api_binint_is_present (surface)."""
import pickle

assert hasattr(pickle, "BININT")
print("api_binint_is_present OK")
