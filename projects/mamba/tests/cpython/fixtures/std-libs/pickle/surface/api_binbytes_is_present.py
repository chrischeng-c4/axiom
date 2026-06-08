# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binbytes_is_present"
# subject = "pickle.BINBYTES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINBYTES: api_binbytes_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINBYTES")
print("api_binbytes_is_present OK")
