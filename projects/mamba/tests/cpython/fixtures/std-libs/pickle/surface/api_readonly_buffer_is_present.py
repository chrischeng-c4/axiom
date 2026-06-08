# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_readonly_buffer_is_present"
# subject = "pickle.READONLY_BUFFER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.READONLY_BUFFER: api_readonly_buffer_is_present (surface)."""
import pickle

assert hasattr(pickle, "READONLY_BUFFER")
print("api_readonly_buffer_is_present OK")
