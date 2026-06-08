# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_inst_is_present"
# subject = "pickle.INST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.INST: api_inst_is_present (surface)."""
import pickle

assert hasattr(pickle, "INST")
print("api_inst_is_present OK")
