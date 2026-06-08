# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_long_binput_is_present"
# subject = "pickle.LONG_BINPUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.LONG_BINPUT: api_long_binput_is_present (surface)."""
import pickle

assert hasattr(pickle, "LONG_BINPUT")
print("api_long_binput_is_present OK")
