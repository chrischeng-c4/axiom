# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_sha3_384_is_present"
# subject = "hashlib.sha3_384"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.sha3_384: api_sha3_384_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "sha3_384")
print("api_sha3_384_is_present OK")
