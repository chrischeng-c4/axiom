# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_stop_is_present"
# subject = "pickle.STOP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.STOP: api_stop_is_present (surface)."""
import pickle

assert hasattr(pickle, "STOP")
print("api_stop_is_present OK")
