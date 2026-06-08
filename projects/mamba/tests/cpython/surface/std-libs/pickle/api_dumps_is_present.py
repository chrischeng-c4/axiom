# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_dumps_is_present"
# subject = "pickle.dumps"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.dumps: api_dumps_is_present (surface)."""
import pickle

assert hasattr(pickle, "dumps")
print("api_dumps_is_present OK")
