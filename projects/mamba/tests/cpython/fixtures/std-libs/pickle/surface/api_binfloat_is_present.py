# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binfloat_is_present"
# subject = "pickle.BINFLOAT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINFLOAT: api_binfloat_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINFLOAT")
print("api_binfloat_is_present OK")
