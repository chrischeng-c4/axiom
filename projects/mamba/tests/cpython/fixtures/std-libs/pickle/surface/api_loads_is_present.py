# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_loads_is_present"
# subject = "pickle.loads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.loads: api_loads_is_present (surface)."""
import pickle

assert hasattr(pickle, "loads")
print("api_loads_is_present OK")
