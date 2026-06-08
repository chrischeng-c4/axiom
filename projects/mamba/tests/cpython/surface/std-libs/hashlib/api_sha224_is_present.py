# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "api_sha224_is_present"
# subject = "hashlib.sha224"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""hashlib.sha224: api_sha224_is_present (surface)."""
import hashlib

assert hasattr(hashlib, "sha224")
print("api_sha224_is_present OK")
