# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binbytes8_is_present"
# subject = "pickle.BINBYTES8"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINBYTES8: api_binbytes8_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINBYTES8")
print("api_binbytes8_is_present OK")
