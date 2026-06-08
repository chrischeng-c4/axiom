# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_next_buffer_is_present"
# subject = "pickle.NEXT_BUFFER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.NEXT_BUFFER: api_next_buffer_is_present (surface)."""
import pickle

assert hasattr(pickle, "NEXT_BUFFER")
print("api_next_buffer_is_present OK")
