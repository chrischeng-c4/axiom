# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binget_is_present"
# subject = "pickle.BINGET"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINGET: api_binget_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINGET")
print("api_binget_is_present OK")
