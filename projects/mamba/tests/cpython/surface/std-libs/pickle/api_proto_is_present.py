# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_proto_is_present"
# subject = "pickle.PROTO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.PROTO: api_proto_is_present (surface)."""
import pickle

assert hasattr(pickle, "PROTO")
print("api_proto_is_present OK")
