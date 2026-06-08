# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_ext2_is_present"
# subject = "pickle.EXT2"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.EXT2: api_ext2_is_present (surface)."""
import pickle

assert hasattr(pickle, "EXT2")
print("api_ext2_is_present OK")
