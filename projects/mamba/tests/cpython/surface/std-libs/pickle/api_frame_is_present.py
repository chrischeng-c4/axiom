# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_frame_is_present"
# subject = "pickle.FRAME"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.FRAME: api_frame_is_present (surface)."""
import pickle

assert hasattr(pickle, "FRAME")
print("api_frame_is_present OK")
