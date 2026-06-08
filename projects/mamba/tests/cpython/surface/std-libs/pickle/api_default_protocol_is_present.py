# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_default_protocol_is_present"
# subject = "pickle.DEFAULT_PROTOCOL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.DEFAULT_PROTOCOL: api_default_protocol_is_present (surface)."""
import pickle

assert hasattr(pickle, "DEFAULT_PROTOCOL")
print("api_default_protocol_is_present OK")
