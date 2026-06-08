# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "surface"
# case = "api_ext4_is_present"
# subject = "pickle.EXT4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pickle.EXT4: api_ext4_is_present (surface)."""
import pickle

assert hasattr(pickle, "EXT4")
print("api_ext4_is_present OK")
