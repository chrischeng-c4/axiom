# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_short_binbytes_is_present"
# subject = "pickle.SHORT_BINBYTES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.SHORT_BINBYTES: api_short_binbytes_is_present (surface)."""
import pickle

assert hasattr(pickle, "SHORT_BINBYTES")
print("api_short_binbytes_is_present OK")
