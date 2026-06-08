# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_unicode_is_present"
# subject = "pickle.UNICODE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.UNICODE: api_unicode_is_present (surface)."""
import pickle

assert hasattr(pickle, "UNICODE")
print("api_unicode_is_present OK")
