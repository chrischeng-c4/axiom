# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_binput_is_present"
# subject = "pickle.BINPUT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.BINPUT: api_binput_is_present (surface)."""
import pickle

assert hasattr(pickle, "BINPUT")
print("api_binput_is_present OK")
