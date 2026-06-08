# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_new_is_present"
# subject = "hashlib.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.new: api_new_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "new")
print("api_new_is_present OK")
