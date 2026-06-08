# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_long4_is_present"
# subject = "pickle.LONG4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.LONG4: api_long4_is_present (surface)."""
import pickle

assert hasattr(pickle, "LONG4")
print("api_long4_is_present OK")
