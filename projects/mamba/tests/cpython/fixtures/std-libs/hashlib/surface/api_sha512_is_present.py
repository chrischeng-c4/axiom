# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_sha512_is_present"
# subject = "hashlib.sha512"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.sha512: api_sha512_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "sha512")
print("api_sha512_is_present OK")
