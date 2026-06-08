# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_dump_is_present"
# subject = "pickle.dump"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.dump: api_dump_is_present (surface)."""
import pickle

assert hasattr(pickle, "dump")
print("api_dump_is_present OK")
