# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_blake2b_is_present"
# subject = "hashlib.blake2b"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.blake2b: api_blake2b_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "blake2b")
print("api_blake2b_is_present OK")
